import os
import subprocess
import time
import sys
import os
#import http.server
#import socketserver

# for nonblocking
import io
import fcntl

# script to build and run all instances in order

'''
# https://stackabuse.com/serving-files-with-pythons-simplehttpserver-module/
class MyHttpRequestHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/':
            self.path = '/home/docweirdo/Documents/docweirdo.github.io/index.html'
        return http.server.SimpleHTTPRequestHandler.do_GET(self)

# Create an object of the above class
handler_object = MyHttpRequestHandler


my_server = socketserver.TCPServer(("127.0.0.80", 8080), handler_object)

# Star the server
my_server.serve_forever()
'''

# https://stackoverflow.com/questions/375427/a-non-blocking-read-on-a-subprocess-pipe-in-python
def set_nonblocking(f: io.BufferedIOBase):
    fl = fcntl.fcntl(f.fileno(), fcntl.F_GETFL)
    fcntl.fcntl(f.fileno(), fcntl.F_SETFL, fl | os.O_NONBLOCK)



# reads stdout of each process and prints it nice with a specified prefix
def poll_stdout(processes):
    for proc in processes:
        try:
            l = proc.stdout.readline()
            if len(l) > 0:
                msg = l.decode().rstrip("\n")
                print(f"[{proc.prefix}] {msg}")
        except:
            pass
 
    


print("Starting all, terminal pipe to stub")

# add switch so is not allways build
build_res = subprocess.run(["cargo", "build"])

if build_res.returncode == 0:
    try:
        dns_servers = []
        # read all dns configs
        for cfg in os.listdir('./server_configs'):
            if not cfg.endswith(".json"):
                print(cfg + " must be json file")
                sys.exit(-1)
                
            server_name = cfg.split('.')[0]
            print(f"starting dns server {server_name}...")
            server = subprocess.Popen(["./target/debug/dns_server", cfg], stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            server.prefix = server_name

            set_nonblocking(server.stdout)

            dns_servers.append(server)


        rec_res = subprocess.Popen(["./target/debug/recursive_resolver"], stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        rec_res.prefix = "recr"
        stub_res = subprocess.Popen(["./target/debug/stub_resolver", "http"], stdin=sys.stdin, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        stub_res.prefix = "stub"


        # dont block if no data available
        set_nonblocking(rec_res.stdout)
        set_nonblocking(stub_res.stdout)


        # read output
        while stub_res.poll() == None:
            
            poll_stdout(dns_servers + [rec_res, stub_res])
            # ugly but keeps python from busy-spinning
            time.sleep(0.001)

    except KeyboardInterrupt:
        print("exit via python")
    finally:

        # just wait for 100 more lines? hacky but works if backtrace is <= 100 lines and present with max 100ms
        for i in range(100):
            time.sleep(0.001)
            poll_stdout(dns_servers + [rec_res, stub_res])
            # prints error messages
        stub_res.terminate()
        rec_res.terminate()
        server.terminate()
        print("all terminated")
    
    
else:
    print("build failed")

