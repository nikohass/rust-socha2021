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
    def __init__(self, client1, client2, results, *args, test_server_path=TEST_SERVER_PATH):
        Thread.__init__(self)
        self.client1 = client1
        self.client2 = client2
        self.test_server_path = test_server_path
        self.args = args + ("-a true",)
        self.stop = False
        self.results = results
        if not os.path.exists(self.client1):
            raise Exception(f"wrong path for client 1: {self.client1}")
        if not os.path.exists(self.client2):
            raise Exception(f"wrong path for client 2: {self.client2}")

    def run(self):
        cmd = f"{self.test_server_path} --one {self.client1} --two {self.client2}"
        for argument in self.args:
            cmd += " " + argument

        p = subprocess.Popen(cmd.split(), stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        while True:
            retcode = p.poll()
            line = p.stdout.readline()
            if line != b"":
                try:
                    self.results.append([int(v) for v in line.strip().split()])
                except Exception as e:
                    print(e)
                    print(line)
            if self.stop or line == b"bye":
                p.terminate()
                break

    def __str__(self):
        return f"TestServer({self.client1}, {self.client2}, {self.args})"

    def __repr__(self):
        return str(self)

def calculate_LOS(wins, draws, losses):
    def erf(x):
        x = abs(x)
        t = 1.0 / (1.0 + 0.3275911 * x)
        y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * math.exp(-x*x)
        return y if x >= 0 else -y
    return 0.5 + 0.5 * erf((wins - losses) / (2.0 * (wins + draws + losses) ** 0.5))

def get_stats(results):
    average_score = 0
    wins = 0
    losses = 0
    draws = 0
    for result in results:
        average_score += result[0]
        if result[0] > 0:
            wins += 1
        elif result == 0:
            draws += 1
        else:
            losses += 1
    average_score = round(average_score / len(results), 2) if len(results) > 0 else None
    LOS = round(calculate_LOS(wins, draws, losses), 2) if len(results) > 0 else None
    return average_score, wins, draws, losses, LOS

def run_tests(client1, client2, servers=3, t=200):
    results = []
    threads = [TestServer(client1, client2, results, f"--time {t}") for _ in range(servers)]
    for thread in threads:
        thread.daemon = True
        print("Starting", thread)
        thread.start()

    last_len = None
    while True:
        if len(results) != last_len:
            last_len = len(results)
            average_score, wins, draws, losses, LOS = get_stats(results)
            print(f"games: {len(results)} average score: {average_score} wins: {wins} draws: {draws} losses: {losses} LOS: {LOS}")

            if len(results) > 30 and LOS > 0.95:
                if wins > losses:
                    print(client1, end=" is the better client\n")
                else:
                    print(client2, end=" is the better client\n")
                break
        time.sleep(1)

    for thread in threads:
        thread.stop = True

client1 = sys.argv[1].strip()
client2 = sys.argv[2].strip()
t = 1900
servers = 3
if len(sys.argv) > 3:
    t = int(sys.argv[3].strip())
if len(sys.argv) > 4:
    servers = int(sys.argv[4].strip())
print(f"Client1: {client1}\nClient2: {client2}\ntime/action: {t}\ntest servers: {servers}")

run_tests(
    PATH + "/target/release/" + client1,
    PATH + "/target/release/" + client2,
    servers=servers,
    t=t
)
