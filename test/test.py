import os
import sys
import time
import math
import subprocess
from threading import Thread

root = os.path.dirname(os.path.dirname(__file__))
os.chdir(root)
TEST_SERVER_PATH = root + "/target/release/test_server.exe"

class TestResult:
    def __init__(self):
        self.one = 0
        self.draws = 0
        self.two = 0
        self.games = 0
        self.sum_results = 0

    @staticmethod
    def from_line(line):
        ret = TestResult()
        ret.one, ret.draws, ret.two, ret.games, ret.sum_results = \
            tuple([int(entry) for entry in line.decode("utf-8").strip().split()])
        return ret

    def serialize(self):
        return f"Result: {self.one} {self.draws} {self.two} {self.games} {self.sum_results}"

    @staticmethod
    def deserialize(string):
        ret = TestResult()
        ret.one, ret.draws, ret.two, ret.games, ret.sum_results = \
            tuple([int(entry) for entry in string[8:].strip().split()])
        return ret

    def get_average_result(self):
        return self.sum_results / self.games if self.games != 0 else 0

    def __add__(self, other):
        ret = TestResult()
        ret.one = self.one + other.one
        ret.draws = self.draws + other.draws
        ret.two = self.two + other.two
        ret.games = self.games + other.games
        ret.sum_results = self.sum_results + other.sum_results
        return ret

    def __str__(self):
        return f"One: {self.one} Draws: {self.draws} Two: {self.two} Average: {round(self.get_average_result(), 2)}"

    def __repr__(self):
        return str(self)

class TestThread(Thread):
    def __init__(self, one: str, two: str, on_result_update, on_exception, args):
        Thread.__init__(self)
        self.daemon = True
        self.one = one
        self.two = two
        self.on_result_update = on_result_update
        self.on_exception = on_exception
        self.args = args
        self.stop = False
        self.test_result = TestResult()
        self.check_paths()

    def check_paths(self):
        if not os.path.exists(self.one):
            raise Exception(f"wrong path for client 1: {self.one}")
        if not os.path.exists(self.one):
            raise Exception(f"wrong path for client 2: {self.two}")

    def run(self):
        cmd = f"{TEST_SERVER_PATH} --one {self.one} --two {self.two}"
        for argument in self.args:
            cmd += " " + argument
        p = subprocess.Popen(cmd.split(), stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        while not self.stop:
            line = p.stdout.readline()
            if line != b"":
                try:
                    self.test_result = TestResult.from_line(line)
                    self.on_result_update()
                except Exception as e:
                    print(e)
                    print(line)
                    self.stop = True
                    self.on_exception()
            if self.stop:
                break
        p.terminate()

    def __str__(self):
        return f"TestThread({self.client1}, {self.client2}, {self.args})"

    def __repr__(self):
        return str(self)

class ThreadManager:
    def __init__(self, one: str, two: str, time: int, n_threads: int, on_result_update, games=-1):
        self.one = one
        self.two = two
        self.time = time
        self.n_threads = n_threads
        self.threads = []
        self.on_result_update = on_result_update
        self.games = games
        self.test_result = TestResult()
        self.running = False

    def start(self):
        self.running = True
        self.threads = [
            TestThread(self.one, self.two, self.on_thread_update, self.on_exception, (f"--time {self.time}", "--games 1000000"))
            for _ in range(self.n_threads)
        ]
        for thread in self.threads:
            time.sleep(3)
            thread.start()

    def on_exception(self):
        for t in self.threads:
            if t.stop:
                self.stop()
                print("A thread stopped")

    def stop(self):
        for thread in self.threads:
            thread.stop = True
        self.running = False

    def get_test_result(self):
        test_result = TestResult()
        for thread in self.threads:
            test_result += thread.test_result
        self.test_result = test_result

    def on_thread_update(self):
        self.get_test_result()
        self.on_result_update(self)
        if self.games != -1 and self.test_result.games > games:
            self.stop()

def on_result_update(tm):
    print(tm.test_result)

if __name__ == "__main__":
    os.chdir("target/release")
    one = sys.argv[1].strip()
    two = sys.argv[2].strip()
    t = 1900
    servers = 3
    if len(sys.argv) > 3:
        t = int(sys.argv[3].strip())
    if len(sys.argv) > 4:
        threads = int(sys.argv[4].strip())
        if threads > 3:
            input(f"Start {threads} testservers?")
    print(f"Client 1: {one}\nClient 2: {two}\nTime/Action: {t}\nThreads: {threads}")
    tm = ThreadManager(one, two, t, threads, on_result_update)
    tm.start()

    try:
        while tm.running:
            time.sleep(1)
    except KeyboardInterrupt:
        print("Canceled")
