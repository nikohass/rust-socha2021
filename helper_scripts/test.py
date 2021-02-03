import os
import sys
import time
import math
import subprocess
from threading import Thread

PATH = os.path.dirname(os.path.dirname(__file__))
os.chdir(PATH)
TEST_SERVER_PATH = PATH + "/target/release/test_server.exe"

class TestServer(Thread):
    def __init__(self, client1, client2, *args):
        Thread.__init__(self)
        self.daemon = True
        self.client1 = client1
        self.client2 = client2
        self.args = args + ("--games 1000000",)
        self.stop = False
        self.result = None
        self.updated = False
        if not os.path.exists(self.client1):
            raise Exception(f"wrong path for client 1: {self.client1}")
        if not os.path.exists(self.client2):
            raise Exception(f"wrong path for client 2: {self.client2}")

    def run(self):
        cmd = f"{TEST_SERVER_PATH} --one {self.client1} --two {self.client2}"
        for argument in self.args:
            cmd += " " + argument

        p = subprocess.Popen(cmd.split(), stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        while True:
            retcode = p.poll()
            line = p.stdout.readline()
            if line != b"":
                try:
                    self.result = [int(entry) for entry in line.decode("utf-8").strip().split()]
                    self.updated = True
                except Exception as e:
                    print(e)
                    print(line)
                    self.stop = True
            if self.stop:
                p.terminate()
                break

    def __str__(self):
        return f"TestServer({self.client1}, {self.client2}, {self.args})"

    def __repr__(self):
        return str(self)

def print_stats(threads):
    stats = [sum([thread.result[i] for thread in threads if thread.result != None]) for i in range(5)]
    one, draws, two, games, sum_results = tuple(stats)
    print(f"Games: {games} Result: {one} {draws} {two} Average result: {round(sum_results / games, 2)}")
    return games

def run_tests(client1, client2, servers=3, t=1900, games=300):
    if games < servers:
        servers = games
    threads = [TestServer(client1, client2, f"--time {t}") for _ in range(servers)]
    for thread in threads:
        thread.start()

    while True:
        if any([thread.updated for thread in threads]):
            if print_stats(threads) > games:
                break
            for thread in threads:
                thread.updated = False
        time.sleep(0.1)

    for thread in threads:
        thread.stop = True

if __name__ == "__main__":
    client1 = sys.argv[1].strip()
    client2 = sys.argv[2].strip()
    t = 1900
    servers = 3
    if len(sys.argv) > 3:
        t = int(sys.argv[3].strip())
    if len(sys.argv) > 4:
        servers = int(sys.argv[4].strip())
        if servers > 3:
            input(f"Start {servers} testservers?")
    print(f"client1: {client1}\nclient2: {client2}\ntime/action: {t}\ntest servers: {servers}")

    run_tests(
        PATH + "/target/release/" + client1,
        PATH + "/target/release/" + client2,
        servers=servers,
        t=t,
    )
