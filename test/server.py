import socket
from threading import Thread
from test import TestResult
import os

PORT = 30_000
VERBOSE = True

root = os.path.dirname(os.path.dirname(__file__))
os.chdir(root)

print(socket.gethostname())
print(socket.gethostbyname(socket.gethostname()))

def log(*args, **kwargs):
    if VERBOSE:
        print(*args, **kwargs)

class ClientInfo:
    def __init__(self, addr):
        self.addr = addr
        self.cores = -1
        self.stop = False
        self.result = TestResult()

class ClientThread(Thread):
    def __init__(self, conn, addr, on_task_request, on_result_update):
        Thread.__init__(self)
        self.daemon = True
        self.conn = conn
        self.on_task_request = on_task_request
        self.on_result_update = on_result_update
        self.info = ClientInfo(addr[0])
        self.start()

    def run(self):
        while True:
            try:
                request = self.conn.recv(4096)
            except ConnectionResetError:
                log(f"{self.info.addr} disconnected")
                break
            self.on_request(request.decode("utf-8"))

    def on_request(self, request):
        entries = request.split()
        request_type = entries[0].lower()
        e = entries[1:]
        if request_type == "init":
            self.handle_init(e)
        elif request_type == "file":
            self.handle_file_request(e)
        elif request_type == "task":
            self.handle_task_request(e)
        elif request_type == "check":
            self.handle_check_request(e)
        elif request_type == "result:":
            self.handle_result_request(request)
        else:
            log("Unknown request type:", request_type)

    def handle_init(self, entries):
        self.info.cores = int(entries[0])
        log(f"{self.info.addr} connected")
        self.conn.send("Ok".encode("utf-8"))

    def handle_file_request(self, entries):
        filename = entries[0].lower()
        log(f"{self.info.addr} requested file \"{filename}\" ", end="")
        if not os.path.exists(filename):
            self.conn.sendall("-1".encode("utf-8"))
            log("Requested file does not exist.")
            return
        log("Sending file... ", end="")
        with open(filename, "rb") as file:
            data = file.read()
        self.conn.sendall(str(len(data)).encode("utf-8"))
        self.conn.sendall(data)
        log("Done")

    def handle_task_request(self, entries):
        log(f"{self.info.addr} requested a task.", end="")
        task = self.on_task_request(self)
        if task == None:
            log(" No task available")
            self.conn.sendall("-1".encode("utf-8"))
            return
        log(" Sending task... ", end="")
        self.conn.sendall(str(len(task)).encode("utf-8"))
        self.conn.sendall(task)
        log("Done")

    def handle_check_request(self, _):
        if self.info.stop:
            self.conn.sendall("stop".encode("utf-8"))
        else:
            self.conn.sendall("ok".encode("utf-8"))

    def handle_result_request(self, request):
        self.info.result = TestResult.deserialize(request)
        self.handle_check_request(request)
        self.on_result_update()

    def __str__(self):
        string = self.info.addr + " " * (15 - len(self.info.addr))
        st = str(self.info.cores)
        string = string + " " * (6 - len(st)) + st
        return string

    def __repr__(self):
        return str(self)

class Server(Thread):
    def __init__(self, task=None):
        Thread.__init__(self)
        self.clients = {}
        self.task = task
        self.result = TestResult()

    def on_task_request(self, client):
        return self.task

    def on_result_update(self):
        result = TestResult()
        for key in self.clients:
            result += self.clients[key].info.result
        log(result)

    def run(self):
        while True:
            self.accept_client()

    def accept_client(self):
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.bind(("", PORT))
        s.listen()
        conn, addr = s.accept()
        self.clients[addr] = ClientThread(conn, addr, self.on_task_request, self.on_result_update)

    def reset_results(self):
        for key in self.clients:
            self.clients[key].info.result = TestResult()

    def stop(self):
        print("Stop")
        for key in self.clients:
            self.clients[key].info.stop = True
        self.task = None

    def test(self):
        print("Test")
        self.reset_results()
        with open("test/tasks/run_test.py", "rb") as file:
            self.task = file.read()
        for key in self.clients:
            self.clients[key].info.stop = False

    def update(self):
        print("Update")
        with open("test/tasks/update.py", "rb") as file:
            self.task = file.read()

    def list_clients(self):
        sum_cores = 0
        print("Address         Cores")
        for key in self.clients:
            print(self.clients[key])
            sum_cores += self.clients[key].info.cores
        print(f"Cores: {sum_cores}")

s = Server()
s.start()

while True:
    i = input().lower()
    if i == "test" or i == "t":
        s.test()
    elif i == "stop" or i == "s":
        s.stop()
    elif i == "list" or i == "l":
        s.list_clients()
    elif i == "update":
        s.update()
