import os
import time

if SERVER_ADDRESS != "localhost":
    for filename in os.listdir("test/"):
        path = "test/" + filename
        if ".py" in path:
            print(f"Updating {filename}")
            c.request_file(path, path)
time.sleep(30)
