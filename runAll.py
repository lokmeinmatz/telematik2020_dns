import os
import subprocess
import time
import threading
import sys
import os


# script to build and run all instances in order


# https://stackoverflow.com/questions/287871/how-to-print-colored-text-in-python
class bcolors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'


# call threaded, listenes to stdout of process and prints with prefix
def poll_subprocess(proc: subprocess.Popen):
    ret = None
    while ret == None:
        ret = proc.poll()
        #print(f"waintig for message from {proc.prefix}")
        msg = proc.stdout.readline().decode().rstrip("\n")
        if len(msg) > 0:
            print(f"[{proc.prefix}] {msg}")
    print(f"{bcolors.WARNING}>>> Process {proc.prefix.ljust(20)} terminated with code {ret}{bcolors.ENDC}")

print("Starting local dns system...")


# optional build project
build_res = None
if "build" in sys.argv:
    # add switch so is not allways build
    print("Building binaries...")
    build_res = subprocess.run(["cargo", "build"])
else:
    # check if binaries exist
    bins = ["./target/debug/dns_server", "./target/debug/recursive_resolver", "./target/debug/stub_resolver"]
    for binpath in bins:
        if not os.path.exists(binpath):
            print(f"Binary {binpath} doesn't exist. Try running python3 runAll.py build to first build the programs.")
            sys.exit(-1)




if build_res == None or build_res.returncode == 0:
    try:
        # read all dns configs and start a server for each
        dns_servers = []
        for cfg in os.listdir('./server_configs'):
            if not cfg.endswith(".json"):
                print(cfg + " must be json file")
                sys.exit(-1)
                
            server_name = cfg.split('.')[0]
            print(f"starting dns server {server_name}...")
            server = subprocess.Popen(["./target/debug/dns_server", cfg], stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            server.prefix = server_name
            dns_servers.append(server)


        rec_res = subprocess.Popen(["./target/debug/recursive_resolver"], stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        rec_res.prefix = "recr"
        stub_res = subprocess.Popen(["./target/debug/stub_resolver", "interactive", "proxy"], stdin=sys.stdin, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        stub_res.prefix = "stub"


        # start a thread for each process to get stdout
        print("Starting stdout threads")
        stdout_threads = []
        for proc in dns_servers + [rec_res, stub_res]:
            t = threading.Thread(target=poll_subprocess, args=[proc])
            t.start()
            stdout_threads.append(t)

        # wait until first thread finsihed, then kill all others
        while True:
            all_alive = True
            for listener in stdout_threads:
                t.join(0.01)
                if not t.is_alive():
                    all_alive = False
            if not all_alive:
                break

    except KeyboardInterrupt:
        print("exit via python")
    finally:

        
        stub_res.terminate()
        rec_res.terminate()
        for server in dns_servers:
            server.terminate()
    
    
else:
    print("build failed")

