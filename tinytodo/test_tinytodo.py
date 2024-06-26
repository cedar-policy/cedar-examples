from tinytodo import *
import time
import unittest
import io
from contextlib import redirect_stdout

# Tests are flakey if the delay after start and stop is too short, 0.1 seconds
# seems to be fine. Tests will fail if there's an instance of the TinyTodo
# server running already. E.g., if you start one in the repl and don't kill it
# before running these tests.
class TinyTodoTest(unittest.TestCase):
    def setUp(self):
        start_server()
        time.sleep(0.1)
        set_user(andrew)

    def tearDown(self):
        stop_server()
        time.sleep(0.1)

    def assert_in_stdout(self, s, f):
        out = io.StringIO()
        with redirect_stdout(out):
            f()
        self.assertIn(s, out.getvalue())

    def test_owner_get_list(self):
        self.assert_in_stdout("Created list ID 0", lambda : create_list("foo"))
        self.assert_in_stdout("=== foo ===", lambda : get_list(0))

    def test_unauthorized_ops(self):
        self.assert_in_stdout("Created list ID 0", lambda : create_list("foo"))
        set_user(emina)
        self.assert_in_stdout("Access denied", lambda : get_list(0))
        self.assert_in_stdout("Access denied", lambda : share_list(0, emina, True))
        self.assert_in_stdout("Access denied", lambda : share_list(0, emina, False))
        self.assert_in_stdout("Access denied", lambda : unshare_list(0, emina))
        self.assert_in_stdout("Access denied", lambda : create_task(0, "bar"))
        self.assert_in_stdout("Access denied", lambda : create_task(0, "bar"))
        self.assert_in_stdout("Access denied", lambda : change_task_description(0, 0, "bar"))
        self.assert_in_stdout("Access denied", lambda : delete_task(0, 0))
        self.assert_in_stdout("Access denied", lambda : delete_list(0))

    def test_revoked_ops(self):
        self.assert_in_stdout("Created list ID 0", lambda : create_list("foo"))
        self.assert_in_stdout("Shared list ID 0 with emina", lambda : share_list(0, emina, True))
        self.assert_in_stdout("Unshared read permissions on list ID 0 with emina", lambda : unshare_list(0, emina))
        set_user(emina)
        self.assert_in_stdout("Access denied", lambda : get_list(0))
        self.assert_in_stdout("Access denied", lambda : share_list(0, emina, True))
        self.assert_in_stdout("Access denied", lambda : share_list(0, emina, False))
        self.assert_in_stdout("Access denied", lambda : unshare_list(0, emina))
        self.assert_in_stdout("Access denied", lambda : create_task(0, "bar"))
        self.assert_in_stdout("Access denied", lambda : create_task(0, "bar"))
        self.assert_in_stdout("Access denied", lambda : change_task_description(0, 0, "bar"))
        self.assert_in_stdout("Access denied", lambda : delete_task(0, 0))
        self.assert_in_stdout("Access denied", lambda : delete_list(0))

    def test_shared_read_only(self):
        self.assert_in_stdout("Created list ID 0", lambda : create_list("foo"))
        self.assert_in_stdout("Shared list ID 0 with emina", lambda : share_list(0, emina, True))
        set_user(emina)
        self.assert_in_stdout("=== foo ===", lambda : get_list(0))
        self.assert_in_stdout("Access denied", lambda : create_task(0, "bar"))

    def test_shared_read_write(self):
        self.assert_in_stdout("Created list ID 0", lambda : create_list("foo"))
        self.assert_in_stdout("Shared list ID 0 with emina", lambda : share_list(0, emina, False))
        set_user(emina)
        self.assert_in_stdout("Created task", lambda : create_task(0, "bar"))
        self.assert_in_stdout("1: [ ] bar", lambda : get_list(0))
        
    def test_get_lists(self):
        self.assert_in_stdout("Created list ID 0", lambda : create_list("foo"))
        self.assert_in_stdout("Created list ID 3", lambda : create_list("bar"))
        self.assert_in_stdout('Lists: foo,bar', lambda: get_lists())
        
