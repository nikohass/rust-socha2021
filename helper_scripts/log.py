import sys
import os
import subprocess
import time

# logs stdout and stderr of the client

def clear_log():
    file = open("helper_scripts/log.txt", "w")
    file.write("")
    file.close()

def log(string):
    file = open("helper_scripts/log.txt", "a")
    file.write(string)
    file.close()

def main():
    path = os.path.dirname(os.path.dirname(__file__))
    cmd = path + "/target/release/xml_client.exe --reservation " + sys.argv[-1].strip()
    env = os.environ
    env["RUST_BACKTRACE"] = "1"
    clear_log()

    p = subprocess.Popen(cmd.split(), stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=env)

    while True:
        time.sleep(0.05)
        retcode = p.poll()
        line = p.stdout.readline()
        if line != b"":
            log(str(line)[2:-1].replace("\\n", "\n"))
        if b"bye" in line:
            p.terminate()
            break

if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        log(str(e))
