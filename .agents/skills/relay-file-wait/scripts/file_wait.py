"""Disposable file channel for the Claude Code waiting spike, not a Relay CLI."""

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import signal
import tempfile
import time
import uuid


def write_json(path, value):
    """Publish only complete JSON, including when the reader polls mid-write."""
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent,
                                         delete=False) as stream:
            temporary = Path(stream.name)
            json.dump(value, stream, ensure_ascii=False)
            stream.write("\n")
        os.replace(temporary, path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def read_json(path):
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (FileNotFoundError, json.JSONDecodeError, UnicodeDecodeError):
        return None
    return value if isinstance(value, dict) else None


def prepare(workspace, prompt, timeout):
    workspace = workspace.resolve(strict=True)
    if not workspace.is_dir() or not prompt.strip():
        raise ValueError("workspace must be a directory and prompt must not be empty")
    if not math.isfinite(timeout) or timeout <= 0:
        raise ValueError("timeout must be a finite positive number of seconds")
    if Path(tempfile.gettempdir()).resolve().is_relative_to(workspace):
        raise ValueError("the system temporary directory must be outside the workspace")
    tag = hashlib.sha256(str(workspace).encode()).hexdigest()[:12]
    directory = Path(tempfile.mkdtemp(prefix=f"relay-file-wait-{tag}-")).resolve()
    question = {
        "id": uuid.uuid4().hex,
        "directory": str(directory),
        "workspace": str(workspace),
        "prompt": prompt,
        "expires_at": time.time() + timeout,
    }
    write_json(directory / "question.json", question)
    return question


def reply(directory, answer):
    question = read_json(directory / "question.json")
    if question is None or not answer.strip():
        raise ValueError("a published question and a nonempty answer are required")
    value = {"id": question["id"], "answer": answer, "written_at_ns": time.time_ns()}
    write_json(directory / "answer.json", value)
    return value


def wait_for_answer(directory, *, clock=time.monotonic, sleep=time.sleep, wall_clock=time.time):
    question = read_json(directory / "question.json")
    if question is None:
        raise ValueError("question.json is missing or malformed")
    previous = read_json(directory / "outcome.json")
    if previous is not None:
        return previous
    deadline = clock() + max(0, question["expires_at"] - wall_clock())
    result = {"id": question["id"]}
    try:
        write_json(directory / "started.json", {"pid": os.getpid(), "started_at_ns": time.time_ns()})
        while True:
            remaining = deadline - clock()
            if remaining <= 0:
                result["status"] = "timeout"
                break
            answer = read_json(directory / "answer.json")
            if (answer is not None and answer.get("id") == question["id"]
                    and isinstance(answer.get("answer"), str) and answer["answer"].strip()):
                result.update(status="answered", answer=answer["answer"],
                              written_at_ns=answer.get("written_at_ns"))
                break
            sleep(min(0.1, remaining))
    except KeyboardInterrupt:
        result["status"] = "interrupted"
    result["observed_at_ns"] = time.time_ns()
    write_json(directory / "outcome.json", result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    create = commands.add_parser("prepare", help="publish one question in a new temporary directory")
    create.add_argument("--workspace", type=Path, required=True)
    create.add_argument("--prompt", required=True)
    create.add_argument("--timeout", type=float, required=True, help="seconds from publication")
    wait = commands.add_parser("wait", help="emit one terminal event, with no progress polling output")
    wait.add_argument("directory", type=Path)
    answer = commands.add_parser("reply", help="operator only: atomically write the human answer")
    answer.add_argument("directory", type=Path)
    answer.add_argument("answer")
    args = parser.parse_args()
    def interrupted(signum, frame):
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM, interrupted)
    try:
        if args.command == "prepare":
            result = prepare(args.workspace, args.prompt, args.timeout)
        elif args.command == "reply":
            result = reply(args.directory, args.answer)
        else:
            result = wait_for_answer(args.directory)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.error(str(error))
    print(json.dumps(result, ensure_ascii=False), flush=True)
    return {"timeout": 124, "interrupted": 130}.get(result.get("status"), 0)


if __name__ == "__main__":
    raise SystemExit(main())
