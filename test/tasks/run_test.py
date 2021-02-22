import os
from test import ThreadManager

ONE = "target/release/client.exe"
TWO = "target/release/clients/current_best.exe"
TIME = 1980

#os.system('shutdown -s')
#THREADS = min(7, THREADS)

if not os.path.exists("target"):
    os.mkdir("target")

if not os.path.exists("target/tmp"):
    os.mkdir("target/tmp")

def delete_file(path):
    while os.path.exists(path):
        try:
            os.remove(path)
        except:
            time.sleep(1)

def on_result_update(tm):
    print(tm.test_result)
    if not tm.running:
        return
    c.send(tm.test_result.serialize().encode("utf-8"))
    if c.recieve().decode("utf-8") == "stop":
        print("Terminating all threads")
        tm.stop()

if c.request_file(ONE, "target/tmp/one.exe") and c.request_file(TWO, "target/tmp/two.exe") and \
        c.request_file("target/release/test_server.exe", "target/release/test_server.exe"):
    print(f"Starting {THREADS} threads")
    tm = ThreadManager("target/tmp/one.exe", "target/tmp/two.exe", TIME, THREADS, on_result_update)
    tm.start()
    time.sleep(1)
    while tm.running:
        time.sleep(1)
else:
    print("Unable to download all needed files. Make sure that the paths in test/tasks/run_test.py are correct.")
    raise FileNotFoundError

delete_file("target/tmp/one.exe")
delete_file("target/tmp/two.exe")
