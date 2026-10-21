"""Short checks for the disposable file-wait spike; no model or long wait."""

import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "skills/relay-file-wait/scripts/file_wait.py"
sys.dont_write_bytecode = True


class Clock:
    def __init__(self, on_sleep=lambda: None):
        self.value = 0.0
        self.on_sleep = on_sleep

    def now(self):
        return self.value

    def sleep(self, seconds):
        self.value += seconds
        self.on_sleep()


class FileWaitTests(unittest.TestCase):
    def test_prepare_command_publishes_question_outside_workspace(self):
        with tempfile.TemporaryDirectory() as workspace:
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "prepare", "--workspace", workspace,
                 "--prompt", "Qual opção?", "--timeout", "5"],
                capture_output=True, text=True, check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            prepared = json.loads(result.stdout)
            directory = Path(prepared["directory"])
            self.addCleanup(shutil.rmtree, directory)
            self.assertFalse(directory.is_relative_to(Path(workspace).resolve()))
            question = json.loads((directory / "question.json").read_text())
            self.assertEqual(question["prompt"], "Qual opção?")
            self.assertEqual(question["id"], prepared["id"])
            self.assertEqual(question["workspace"], str(Path(workspace).resolve()))
            self.assertFalse((directory / "answer.json").exists())

    def setUp(self):
        if self._testMethodName == "test_prepare_command_publishes_question_outside_workspace":
            return
        spec = importlib.util.spec_from_file_location("file_wait", SCRIPT)
        self.helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.helper)
        self.workspace = tempfile.TemporaryDirectory()
        self.addCleanup(self.workspace.cleanup)
        self.question = self.helper.prepare(Path(self.workspace.name), "Qual opção?", 5)
        self.directory = Path(self.question["directory"])
        self.addCleanup(shutil.rmtree, self.directory)

    def wait(self, clock=None):
        clock = clock or Clock()
        return self.helper.wait_for_answer(
            self.directory, clock=clock.now, sleep=clock.sleep,
            wall_clock=lambda: self.question["expires_at"] - 1,
        )

    def test_matching_answer_is_returned_and_saved_without_a_sleep(self):
        self.helper.reply(self.directory, "opção B — ação")
        clock = Clock(lambda: self.fail("a valid answer must not wait"))
        result = self.wait(clock)
        self.assertEqual(result["status"], "answered")
        self.assertEqual(result["answer"], "opção B — ação")
        self.assertEqual(result["id"], self.question["id"])
        self.assertEqual(json.loads((self.directory / "outcome.json").read_text()), result)

    def test_missing_answer_expires_without_inventing_one(self):
        result = self.wait()
        self.assertEqual(result["status"], "timeout")
        self.assertNotIn("answer", result)

    def test_partial_wrong_id_and_empty_answers_are_not_consumed(self):
        for answer in ["{", "[]", '{"id":"old","answer":"wrong"}',
                       json.dumps({"id": self.question["id"], "answer": "  "})]:
            with self.subTest(answer=answer):
                (self.directory / "answer.json").write_text(answer)
                clock = Clock(lambda: self.helper.reply(self.directory, "correct"))
                result = self.wait(clock)
                self.assertGreater(clock.value, 0)
                self.assertEqual(result["answer"], "correct")
                (self.directory / "outcome.json").unlink()

    def test_expired_wait_is_not_reopened_by_a_late_answer(self):
        first = self.wait()
        self.helper.reply(self.directory, "late")
        self.assertEqual(self.wait(), first)

    def test_wall_clock_changes_do_not_extend_a_running_wait(self):
        clock = Clock()
        calls = []
        def wall_clock():
            calls.append(True)
            self.assertEqual(len(calls), 1)
            return self.question["expires_at"] - 1
        result = self.helper.wait_for_answer(
            self.directory, clock=clock.now, sleep=clock.sleep, wall_clock=wall_clock,
        )
        self.assertEqual(result["status"], "timeout")
        self.assertAlmostEqual(clock.value, 1)

    def test_interruption_is_terminal_and_has_no_answer(self):
        def interrupt():
            raise KeyboardInterrupt
        result = self.wait(Clock(interrupt))
        self.assertEqual(result["status"], "interrupted")
        self.assertNotIn("answer", result)
        self.helper.reply(self.directory, "late")
        self.assertEqual(self.wait(), result)

    def test_separate_questions_cannot_consume_each_others_answer(self):
        other = self.helper.prepare(Path(self.workspace.name), "Outra?", 5)
        other_dir = Path(other["directory"])
        self.addCleanup(shutil.rmtree, other_dir)
        self.assertNotEqual(self.question["id"], other["id"])
        self.assertNotEqual(self.directory, other_dir)
        self.helper.reply(other_dir, "other")
        shutil.copyfile(other_dir / "answer.json", self.directory / "answer.json")
        result = self.wait()
        self.assertEqual(result["status"], "timeout")

    def test_nonpositive_or_nonfinite_timeout_is_rejected(self):
        for timeout in [0, -1, float("inf"), float("nan")]:
            with self.subTest(timeout=timeout), self.assertRaises(ValueError):
                self.helper.prepare(Path(self.workspace.name), "Pergunta?", timeout)

    def test_cli_timeout_emits_one_terminal_event_and_exit_124(self):
        question = self.question.copy()
        question["expires_at"] = 0
        self.helper.write_json(self.directory / "question.json", question)
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "wait", str(self.directory)],
            capture_output=True, text=True, timeout=10, check=False,
        )
        self.assertEqual(result.returncode, 124, result.stderr)
        self.assertEqual(len(result.stdout.splitlines()), 1)
        self.assertEqual(json.loads(result.stdout)["status"], "timeout")

    def test_cli_sigterm_emits_interrupted_and_exit_130(self):
        process = subprocess.Popen(
            [sys.executable, str(SCRIPT), "wait", str(self.directory)],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        )
        try:
            watchdog = time.monotonic() + 10
            while not (self.directory / "started.json").exists():
                if process.poll() is not None or time.monotonic() >= watchdog:
                    self.fail("waiter did not publish readiness")
                time.sleep(0.01)
            process.terminate()
            stdout, stderr = process.communicate(timeout=10)
            self.assertEqual(process.returncode, 130, stderr)
            self.assertEqual(len(stdout.splitlines()), 1)
            self.assertEqual(json.loads(stdout)["status"], "interrupted")
        finally:
            if process.poll() is None:
                process.kill()
            process.communicate()


if __name__ == "__main__":
    unittest.main()
