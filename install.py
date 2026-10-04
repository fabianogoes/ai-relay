#!/usr/bin/env python3
"""Install Relay skills in the current project and keep relay-tui current."""

import argparse
import hashlib
import io
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path, PurePosixPath


REPOSITORY = "fabianogoes/ai-relay"
API_ROOT = "https://api.github.com/repos/" + REPOSITORY
RAW_ROOT = "https://raw.githubusercontent.com/" + REPOSITORY
USER_AGENT = "relay-install/1"


class InstallError(Exception):
    """An installation error with a message suitable for the terminal."""


def fetch_url(url):
    request = urllib.request.Request(
        url,
        headers={"User-Agent": USER_AGENT, "Accept": "application/vnd.github+json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.read()
    except (urllib.error.URLError, TimeoutError) as error:
        raise InstallError("Download failed for {}: {}".format(url, error)) from error


def fetch_json(url, get=fetch_url):
    try:
        return json.loads(get(url).decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise InstallError("GitHub returned invalid JSON from {}".format(url)) from error


def download_skill_files(commit, entries, get=fetch_url):
    """Fetch only regular files below skills/relay-* at an immutable commit."""
    files = {}
    for entry in entries:
        path = entry.get("path", "")
        if entry.get("type") != "blob" or entry.get("mode") not in ("100644", "100755"):
            continue
        if not path.startswith("skills/"):
            continue
        relative = path[len("skills/"):]
        parts = PurePosixPath(relative).parts
        if len(parts) < 2 or not parts[0].startswith("relay-"):
            continue
        if PurePosixPath(relative).is_absolute() or ".." in parts or "\\" in relative:
            raise InstallError("Unsafe skill path in Relay repository: {}".format(path))
        source_sha = entry.get("sha")
        if not isinstance(source_sha, str) or not re.fullmatch(r"[0-9a-f]{40}", source_sha):
            raise InstallError("GitHub returned no valid blob checksum for {}".format(path))
        url = "{}/{}/{}".format(RAW_ROOT, commit, urllib.parse.quote(path, safe="/"))
        payload = get(url)
        git_blob = "blob {}\0".format(len(payload)).encode("ascii") + payload
        if hashlib.sha1(git_blob).hexdigest() != source_sha:
            raise InstallError("Downloaded skill file did not match its Git commit: {}".format(path))
        files[relative] = (payload, 0o755 if entry["mode"] == "100755" else 0o644)

    skills = {PurePosixPath(path).parts[0] for path in files}
    for skill in skills:
        if skill + "/SKILL.md" not in files:
            raise InstallError("Published skill {} has no SKILL.md".format(skill))
    if not files:
        raise InstallError("No Relay skills were found on GitHub")
    return files


def _validate_skill_files(files):
    skills = {}
    for relative, file_data in files.items():
        path = PurePosixPath(relative)
        if path.is_absolute() or ".." in path.parts or "\\" in relative:
            raise InstallError("Unsafe skill path: {}".format(relative))
        if len(path.parts) < 2 or not path.parts[0].startswith("relay-"):
            raise InstallError("Not a Relay skill path: {}".format(relative))
        payload, mode = file_data
        if not isinstance(payload, bytes) or mode not in (0o644, 0o755):
            raise InstallError("Invalid file data for skill path: {}".format(relative))
        skills.setdefault(path.parts[0], []).append((path, payload, mode))
    for skill, entries in skills.items():
        if not any(path.as_posix() == skill + "/SKILL.md" for path, _, _ in entries):
            raise InstallError("Published skill {} has no SKILL.md".format(skill))
    if not skills:
        raise InstallError("No Relay skills were found")
    return skills


def install_skills(files, project_dir):
    """Atomically replace packaged Relay skill directories, preserving others."""
    skills = _validate_skill_files(files)
    project_dir = Path(project_dir).resolve()
    claude_dir = project_dir / ".claude"
    skills_dir = claude_dir / "skills"
    if claude_dir.is_symlink() or skills_dir.is_symlink():
        raise InstallError("Refusing to install through a symlink at .claude or .claude/skills")
    skills_dir.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix=".relay-install-", dir=str(skills_dir)) as temporary:
        temporary = Path(temporary)
        staged = temporary / "staged"
        backups = temporary / "backups"
        staged.mkdir()
        backups.mkdir()

        for skill, entries in skills.items():
            for relative, payload, mode in entries:
                destination = staged / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(payload)
                destination.chmod(mode)

        installed = []
        try:
            for skill in sorted(skills):
                source = staged / skill
                destination = skills_dir / skill
                backup = backups / skill
                if destination.exists() or destination.is_symlink():
                    os.replace(str(destination), str(backup))
                try:
                    os.replace(str(source), str(destination))
                except OSError:
                    if backup.exists() or backup.is_symlink():
                        os.replace(str(backup), str(destination))
                    raise
                installed.append((destination, backup))
        except OSError as error:
            for destination, backup in reversed(installed):
                if destination.exists() or destination.is_symlink():
                    if destination.is_dir() and not destination.is_symlink():
                        shutil.rmtree(str(destination))
                    else:
                        destination.unlink()
                if backup.exists() or backup.is_symlink():
                    os.replace(str(backup), str(destination))
            raise InstallError("Could not install Relay skills: {}".format(error)) from error

    return sorted(skills)


def verify_checksum(payload, checksum_file):
    """Verify a GitHub Release SHA-256 sidecar before using its asset."""
    try:
        expected = checksum_file.decode("ascii").strip().split()[0].lower()
    except (UnicodeDecodeError, IndexError) as error:
        raise InstallError("The published checksum file is invalid") from error
    if not re.fullmatch(r"[0-9a-f]{64}", expected):
        raise InstallError("The published checksum is not a SHA-256 digest")
    actual = hashlib.sha256(payload).hexdigest()
    if actual != expected:
        raise InstallError("Checksum mismatch; the downloaded file was not installed")


def version_tuple(version):
    match = re.fullmatch(r"v?(\d+)\.(\d+)\.(\d+)", version.strip())
    if not match:
        raise InstallError("Unrecognized relay-tui version: {}".format(version))
    return tuple(int(part) for part in match.groups())


def needs_update(current, latest):
    if current is None:
        return True
    return version_tuple(current) < version_tuple(latest)


def install_tui_binary(binary, home):
    """Atomically put an executable relay-tui in the user's local bin."""
    destination = Path(home) / ".local" / "bin" / "relay-tui"
    destination.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=".relay-tui-", dir=str(destination.parent))
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(binary)
        os.chmod(temporary, 0o755)
        os.replace(temporary, str(destination))
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)
    return destination


def _latest_skills_source(get=fetch_url):
    ref = fetch_json(API_ROOT + "/git/ref/heads/main", get)
    commit = ref.get("object", {}).get("sha")
    if not isinstance(commit, str) or not re.fullmatch(r"[0-9a-f]{40,64}", commit):
        raise InstallError("Could not resolve the current Relay skills revision")
    tree = fetch_json(API_ROOT + "/git/trees/{}?recursive=1".format(commit), get)
    if tree.get("truncated"):
        raise InstallError("GitHub returned an incomplete Relay file list")
    return commit, download_skill_files(commit, tree.get("tree", []), get)


def _release_asset(release, filename):
    for asset in release.get("assets", []):
        if asset.get("name") == filename:
            return asset.get("browser_download_url")
    raise InstallError("The latest relay-tui release has no {} asset".format(filename))


def _platform_target():
    system = platform.system().lower()
    machine = platform.machine().lower()
    targets = {
        ("darwin", "arm64"): "aarch64-apple-darwin",
        ("darwin", "aarch64"): "aarch64-apple-darwin",
        ("darwin", "x86_64"): "x86_64-apple-darwin",
        ("linux", "x86_64"): "x86_64-unknown-linux-musl",
        ("linux", "aarch64"): "aarch64-unknown-linux-musl",
        ("linux", "arm64"): "aarch64-unknown-linux-musl",
    }
    try:
        return targets[(system, machine)]
    except KeyError as error:
        raise InstallError("No relay-tui release is available for {} {}".format(system, machine)) from error


def _current_tui():
    executable = shutil.which("relay-tui")
    if not executable:
        return None, None
    try:
        result = subprocess.run([executable, "--version"], check=True, text=True, capture_output=True, timeout=5)
    except (OSError, subprocess.SubprocessError) as error:
        raise InstallError("Could not read the installed relay-tui version: {}".format(error)) from error
    match = re.search(r"\brelay-tui\s+v?(\d+\.\d+\.\d+)\b", result.stdout + result.stderr)
    if not match:
        raise InstallError("Could not parse relay-tui --version output")
    return match.group(1), Path(executable)


def _binary_from_archive(archive_bytes, expected_name):
    try:
        with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode="r:gz") as archive:
            matches = []
            for member in archive.getmembers():
                path = PurePosixPath(member.name)
                if path.is_absolute() or ".." in path.parts or not member.isfile():
                    continue
                if path.name == expected_name:
                    matches.append(member)
            if len(matches) != 1:
                raise InstallError("The relay-tui archive does not contain exactly one binary")
            stream = archive.extractfile(matches[0])
            if stream is None:
                raise InstallError("Could not read relay-tui from its archive")
            return stream.read()
    except (tarfile.TarError, OSError) as error:
        raise InstallError("The relay-tui archive is invalid: {}".format(error)) from error


def _prepare_tui(get=fetch_url, current=None):
    release = fetch_json(API_ROOT + "/releases/latest", get)
    tag = release.get("tag_name", "")
    match = re.fullmatch(r"relay-tui-v(\d+\.\d+\.\d+)", tag)
    if not match or release.get("draft") or release.get("prerelease"):
        raise InstallError("GitHub's latest release is not a stable relay-tui release")
    latest = match.group(1)
    if not needs_update(current, latest):
        return latest, None
    target = _platform_target()
    archive_name = "relay-tui-{}-{}.tar.gz".format(latest, target)
    archive_url = _release_asset(release, archive_name)
    checksum_url = _release_asset(release, archive_name + ".sha256")
    archive = get(archive_url)
    verify_checksum(archive, get(checksum_url))
    return latest, _binary_from_archive(archive, "relay-tui")


def _run(args):
    parser = argparse.ArgumentParser(
        description="Install Relay skills in this project and install/update relay-tui."
    )
    parser.add_argument("--skills-only", action="store_true", help="install skills without checking relay-tui")
    options = parser.parse_args(args)
    project = Path.cwd()
    print("Installing Relay skills in {}".format(project))

    commit, files = _latest_skills_source()
    tui_version = None
    tui_binary = None
    if not options.skills_only:
        current, current_path = _current_tui()
        tui_version, tui_binary = _prepare_tui(current=current)
        if current is None:
            print("relay-tui is not installed; installing {}".format(tui_version))
        elif tui_binary is None:
            print("relay-tui {} is already current".format(current))
        else:
            print("Updating relay-tui {} to {}".format(current, tui_version))
            if current_path and ".local/bin" not in current_path.as_posix():
                print(
                    "Note: the older command is at {}; the Relay copy will be installed in ~/.local/bin.".format(
                        current_path
                    )
                )

    try:
        installed = install_skills(files, project)
    except OSError as error:
        raise InstallError("Could not write skills in {}: {}".format(project, error)) from error
    print("Installed {} skills from commit {}: {}".format(len(installed), commit[:12], ", ".join(installed)))
    print("Start a new Claude Code session, then run /relay-setup in this project.")

    if tui_binary is not None:
        try:
            executable = install_tui_binary(tui_binary, Path.home())
        except OSError as error:
            print("Relay skills are installed, but relay-tui could not be updated.", file=sys.stderr)
            raise InstallError("Could not write ~/.local/bin/relay-tui: {}".format(error)) from error
        print("Installed relay-tui {} at {}".format(tui_version, executable))
        if shutil.which("relay-tui") != str(executable):
            print("Add ~/.local/bin to PATH to run relay-tui and relay-tui-split.")


def main(args=None):
    try:
        _run(sys.argv[1:] if args is None else args)
    except InstallError as error:
        print("relay install: {}".format(error), file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("relay install: interrupted", file=sys.stderr)
        return 130
    return 0


if __name__ == "__main__":
    sys.exit(main())
