# telematik2020_dns

## to run

### automatic
`python3 runAll.py`

test usage: `python3 runAll.py --proxy`

Now you can enter a domain inside the terminal or connect to the proxy
on `127.0.0.1:8100` and request `router.telematik` to get the index.html served.

#### Flags
- `--build` to first build binaries via cargo
- `--proxy` to enable http proxy and serve `index.html` for `http://router.telematik`


#### what happens when runnimg runAll.py?
- starts an dns server for each file in `server_configs`
- stdin is piped to stub resolver
- when exiting stub or by keyboard interrupt, all get terminated

! runAll.py currently builds as debug build, may be unperformant

### manual
1. build with `cargo build`
2. start the three binaries (in seperate shells)
 - DNS server: `./target/debug/dns_server <config_name>.json`
 - recursive resolver: `./target/debug/recursive_resolver`
 - stub: `./target/debug/stub_resolver interactive proxy` (or leave out either one to only use one mode)
3. enter domain into stub resolver

! rec expects ROOT server on "127.0.0.100", so the root server must bind there

## structure

one main lib: `shared`

3 binary projects, each for the specific job

## folders

### shared

main struct: `DNSPacket`

can be send via `send_dns_packet()`, where the delay is configured

cen get received and deserialized via `recv_dns_packet()`

### dns_server

loads `ServerConfig` from `./server_configs/` and the json file specified as the first argument

listenes for dns requests and checks wether they are in it's zome / delegated area
sends response and logs data

### logs
contains a log file for each server, with zone name and ip as name

### recursive resolver

accepts the requests from the stub
checks in its cache map if the A record was allready stored and TTL isn't expired yet
sends cache ip or iteratively searches nameserver until A record is found or an error occured.
The root server address is fixed in shared::ROOT_SERVER_ADDR (127.0.0.100)

### server_configs

a dns server gets started for each file here

format:

```
{
    "ip": "own ip to bind to",
    "zone": {
        "subdomain": "a-record-ip", ...
    },
    "delegated": {
        "subdomain": "ns-record-ip", ...
    }
}
```


### stub_resolver

the stub and proxy binary
accepts flags
- `proxy` to enable http proxy on 127.0.0.1:8100
- `interactive` to let user enter domain in the terminal