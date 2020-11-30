import os
import subprocess
import time
import sys
import os

# for nonblocking
import io
import fcntl

# script to build and run all instances in order


# https://stackoverflow.com/questions/375427/a-non-blocking-read-on-a-subprocess-pipe-in-python
def set_nonblocking(f: io.BufferedIOBase):
    fl = fcntl.fcntl(f.fileno(), fcntl.F_GETFL)
    fcntl.fcntl(f.fileno(), fcntl.F_SETFL, fl | os.O_NONBLOCK)





print("Starting all, terminal pipe to stub")

build_res = subprocess.run(["cargo", "build"])

if build_res.returncode == 0:
    try:
        server = subprocess.Popen(["./target/debug/dns_server"], stdout=subprocess.PIPE)
        rec_res = subprocess.Popen(["./target/debug/recursive_resolver"], stdout=subprocess.PIPE)
        stub_res = subprocess.Popen(["./target/debug/stub_resolver"], stdin=sys.stdin, stdout=subprocess.PIPE)

        set_nonblocking(server.stdout)
        set_nonblocking(rec_res.stdout)
        set_nonblocking(stub_res.stdout)


        # prefix all output with [stub], [recr], or [dnss]
       
        while stub_res.poll() == None:
            try:
                l = server.stdout.readline()
                if len(l) > 0:
                    print("[dnss] " + l.decode())
            except:
                pass
            try:
                l = rec_res.stdout.readline()
                if len(l) > 0:
                    print("[recr] " + l.decode())
            except:
                pass
            try:
                l = stub_res.stdout.readline()
                if len(l) > 0:
                    print("[stub] " + l.decode())
            except:
                pass


            # ugly but keeps python from busy-spinning
            time.sleep(0.001)

    except KeyboardInterrupt:
        print("exit via python")
    finally:
        stub_res.terminate()
        rec_res.terminate()
        server.terminate()
        print("all terminated")
    
    
else:
    print("build failed")

