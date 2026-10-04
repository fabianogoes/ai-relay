import importlib.util
import hashlib
import io
import json
import tarfile
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("relay_install", ROOT / "install.py")
relay_install = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(relay_install)


class InstallerTests(unittest.TestCase):
    def test_reinstalls_relay_skills_and_preserves_other_project_skills(self):
        with tempfile.TemporaryDirectory() as temporary:
            project = Path(temporary)
            skills = project / ".claude" / "skills"
            (skills / "relay-setup").mkdir(parents=True)
            (skills / "relay-setup" / "old.md").write_text("old")
            (skills / "my-skill").mkdir()
            (skills / "my-skill" / "SKILL.md").write_text("keep")

            files = {
                "relay-setup/SKILL.md": (b"new setup", 0o644),
                "relay-tui-split/SKILL.md": (b"split", 0o644),
                "relay-tui-split/scripts/open-split.sh": (b"#!/bin/sh\n", 0o755),
            }
            relay_install.install_skills(files, project)
            relay_install.install_skills(files, project)

            self.assertEqual((skills / "relay-setup" / "SKILL.md").read_bytes(), b"new setup")
            self.assertFalse((skills / "relay-setup" / "old.md").exists())
            self.assertEqual((skills / "relay-tui-split" / "scripts" / "open-split.sh").stat().st_mode & 0o777, 0o755)
            self.assertEqual((skills / "my-skill" / "SKILL.md").read_text(), "keep")

    def test_rejects_skill_paths_outside_relay_skill_directories(self):
        with tempfile.TemporaryDirectory() as temporary:
            project = Path(temporary)
            with self.assertRaises(relay_install.InstallError):
                relay_install.install_skills({"relay-setup/../../outside": (b"bad", 0o644)}, project)

    def test_downloads_only_relay_skill_files_from_a_commit_tree(self):
        blob_sha = hashlib.sha1(b"blob 8\0contents").hexdigest()
        entries = [
            {"path": "skills/relay-status/SKILL.md", "mode": "100644", "type": "blob", "sha": blob_sha},
            {"path": "skills/relay-tui-split/SKILL.md", "mode": "100644", "type": "blob", "sha": blob_sha},
            {"path": "skills/relay-tui-split/scripts/open-split.sh", "mode": "100755", "type": "blob", "sha": blob_sha},
            {"path": "app/relay-tui/src/main.rs", "mode": "100644", "type": "blob", "sha": blob_sha},
            {"path": "skills/other/SKILL.md", "mode": "100644", "type": "blob", "sha": blob_sha},
        ]
        fetched = []

        def get(url):
            fetched.append(url)
            return b"contents"

        files = relay_install.download_skill_files("commit123", entries, get)

        self.assertEqual(set(files), {
            "relay-status/SKILL.md",
            "relay-tui-split/SKILL.md",
            "relay-tui-split/scripts/open-split.sh",
        })
        self.assertTrue(all("/commit123/skills/" in url for url in fetched))
        self.assertEqual(files["relay-tui-split/scripts/open-split.sh"][1], 0o755)

    def test_rejects_a_skill_file_that_does_not_match_its_commit_hash(self):
        entry = {
            "path": "skills/relay-status/SKILL.md",
            "mode": "100644",
            "type": "blob",
            "sha": "0" * 40,
        }
        with self.assertRaises(relay_install.InstallError):
            relay_install.download_skill_files("commit123", [entry], lambda _: b"tampered")

    def test_checksum_mismatch_is_rejected(self):
        with self.assertRaises(relay_install.InstallError):
            relay_install.verify_checksum(b"archive", b"0" * 64 + b"  package.tar.gz\n")

    def test_tui_version_comparison_never_downgrades(self):
        self.assertTrue(relay_install.needs_update("0.1.0", "0.2.0"))
        self.assertFalse(relay_install.needs_update("0.2.0", "0.2.0"))
        self.assertFalse(relay_install.needs_update("0.3.0", "0.2.0"))

    def test_tui_release_asset_is_verified_before_binary_is_returned(self):
        payload = b"test executable"
        archive_buffer = io.BytesIO()
        with tarfile.open(fileobj=archive_buffer, mode="w:gz") as archive:
            member = tarfile.TarInfo("relay-tui-0.2.0/relay-tui")
            member.size = len(payload)
            member.mode = 0o755
            archive.addfile(member, io.BytesIO(payload))
        archive_bytes = archive_buffer.getvalue()
        archive_name = "relay-tui-0.2.0-aarch64-apple-darwin.tar.gz"
        archive_url = "https://assets.example/" + archive_name
        checksum_url = archive_url + ".sha256"
        release = {
            "tag_name": "relay-tui-v0.2.0",
            "draft": False,
            "prerelease": False,
            "assets": [
                {"name": archive_name, "browser_download_url": archive_url},
                {"name": archive_name + ".sha256", "browser_download_url": checksum_url},
            ],
        }
        files = {
            relay_install.API_ROOT + "/releases/latest": json.dumps(release).encode(),
            archive_url: archive_bytes,
            checksum_url: (hashlib.sha256(archive_bytes).hexdigest() + "  " + archive_name).encode(),
        }

        version, binary = relay_install._prepare_tui(files.__getitem__, current="0.1.0")

        self.assertEqual(version, "0.2.0")
        self.assertEqual(binary, payload)

    def test_installs_tui_to_user_bin_as_executable(self):
        with tempfile.TemporaryDirectory() as temporary:
            executable = relay_install.install_tui_binary(b"binary", Path(temporary))
            self.assertEqual(executable, Path(temporary) / ".local" / "bin" / "relay-tui")
            self.assertEqual(executable.read_bytes(), b"binary")
            self.assertTrue(executable.stat().st_mode & 0o111)


if __name__ == "__main__":
    unittest.main()
