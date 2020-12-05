# telematik2020_dns

Needed:
- Cargo / rust (preferrably via rustup)
- python (optional) for auto run

## to build all projects
`cargo build` in this folder --> all 3 get built into ./target/debug/

## to run

### automatic
`python3 runAll.py`

or `python3 runAll.py build` to first build binaries

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

## structure

one main lib: `shared`

3 binary projects, each for the specific job

### shared

main struct: `DNSPacket`

can be send via `send_dns_packet()`

cen get received and deserialized via `recv_dns_packet()`

## TODO

- authorative flag 
- http proxy in stub resolver