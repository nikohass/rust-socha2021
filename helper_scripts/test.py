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
        self.one_wins = 0
        self.draws = 0
        self.two_wins = 0
        self.sum_one_scores = 0
        self.sum_two_scores = 0

    def __str__(self):
        games = self.one_wins + self.draws + self.two_wins
        return f"Game {games} One: {self.one_wins} Draws: {self.draws} Two: {self.two_wins} One average: {round(self.sum_one_scores / games, 2)} Two average: {round(self.sum_two_scores / games, 2)}"

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
        self.check_paths()

    def check_paths(self):
        if not os.path.exists(self.one):
            raise Exception(f"Wrong path for client 1: {self.one}")
        if not os.path.exists(self.one):
            raise Exception(f"Wrong path for client 2: {self.two}")

    def run(self):
        cmd = f"{TEST_SERVER_PATH} --one {self.one} --two {self.two}"
        for argument in self.args:
            cmd += " " + argument
        p = subprocess.Popen(cmd.split(), stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        while not self.stop:
            line = p.stdout.readline().decode("utf-8")
            if line != b"":
                if line[:8] == "result: ":
                    entries = [float(e) for e in line[8:].split()]
                    self.on_result_update(entries)
                elif "warning" in line:
                    print(line.replace("\\n", ""), end="")
                    #self.stop = True
                    #self.on_exception()
                elif not "info" in line:
                    print(line.replace("\\n", ""), end="")
                    self.stop = True
                    self.on_exception()
                #else:
                    #print(line.strip())
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
        self.running = False
        self.test_result = TestResult()

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

    def stop(self):
        for thread in self.threads:
            thread.stop = True
        self.running = False

    def on_thread_update(self, entries):
        first, result, one_score, two_score = tuple(entries)
        if first != 0:
            result = -result
            one_score, two_score = (two_score, one_score)
        if result > 0:
            self.test_result.one_wins += 1
        elif result < 0:
            self.test_result.two_wins += 1
        else:
            self.test_result.draws += 1
        self.test_result.sum_one_scores += one_score
        self.test_result.sum_two_scores += two_score
        self.on_result_update(self)

def main():
    def on_result_update(tm):
        print(tm.test_result)

    one = "clients/" + sys.argv[1].strip()
    two = "clients/" + sys.argv[2].strip()
    if one.find(".exe") == -1:
        one = "target/release/client.exe"
    if two.find(".exe") == -1:
        two = "target/release/client.exe"
    t = 1600
    threads = 10
    if len(sys.argv) > 3:
        t = int(sys.argv[3].strip())
    if len(sys.argv) > 4:
        threads = int(sys.argv[4].strip())
        if threads > 10:
            input(f"Start {threads} testservers?")

    print(f"One: {one}\nTwo: {two}\nTime/Action: {t}\nThreads: {threads}")
    tm = ThreadManager(one, two, t, threads, on_result_update)
    tm.start()
    try:
        while tm.running:
            time.sleep(1)
    except KeyboardInterrupt:
        print("Canceled")

if __name__ == "__main__":
    main()
