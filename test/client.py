import socket
import time
import multiprocessing
import os

def get_server_address():
    hostname = socket.gethostname()
    ip = socket.gethostbyname(hostname)
    print(hostname, ip)
    address = input("Server Address: ")
    if address == "":
        return "localhost"
    if not "." in address:
        entries = ip.split(".")
        for i in range(len(entries) - 1):
            address = f"{entries[i]}.{address}"
    print(address)
    return address

SERVER_ADDRESS = get_server_address()
PORT = 30_000
CORES = multiprocessing.cpu_count()
THREADS = int(CORES * 0.75)
print(f"Threads: {THREADS}")

root = os.path.dirname(os.path.dirname(__file__))
os.chdir(root)

class Client:
    def __init__(self):
        self.connect()
        self.send(f"init {CORES}".encode("utf-8"))
        self.recieve()

    def connect(self):
        self.s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.s.connect((SERVER_ADDRESS, PORT))

    def send(self, data):
        self.s.sendall(data)

    def recieve(self):
        return self.s.recv(1024)

    def recieve_data(self, size):
        data = bytearray()
        recieved = 0
        while recieved < size:
            chunk = self.s.recv(min(size - recieved, 1024))
            data += chunk
            recieved += len(chunk)
        return data

    def request_file(self, filename, save_as):
        print(f"Requesting \"{filename}\" from {SERVER_ADDRESS}. ", end="")
        self.send(f"file {filename}".encode("utf-8"))
        response = self.recieve().decode("utf-8")
        response = int(response)
        if response == -1:
            print("File not found.")
            return False
        size = response
        print(f"Downloading... ({size} bytes) ", end="")
        data = self.recieve_data(size)
        if size != len(data):
            print("Error")
        with open(save_as, "wb") as file:
            file.write(data)
        print("Done")
        return True

    def request_task(self):
        print(f"Requesting task.", end="")
        self.send("task".encode("utf-8"))
        size = int(self.recieve())
        if size == -1:
            print(" No task available")
            return False
        print(" Downloading task...")
        task = self.recieve_data(size).decode("utf-8")
        try:
            print("Running task")
            exec(task)
        except KeyboardInterrupt:
            raise KeyboardInterrupt
        except Exception as e:
            print("Task did not run successfull", e)
            return False
        return True

    def check(self):
        self.send("check".encode("utf-8"))
        response = self.recieve().decode("utf-8")
        if response == "ok":
            return False
        if response == "stop":
            return True

while True:
    try:
        c = Client()
        if not c.request_task():
            print("Retry in 10s")
            time.sleep(10)
    except KeyboardInterrupt:
        break
    except Exception as e:
        print(e)
        time.sleep(10)
