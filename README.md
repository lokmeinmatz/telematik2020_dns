# telematik2020_dns

Needed:
- Cargo / rust (preferrably via rustup)
- python (optional) for auto run

## to build all projects
`cargo build` in this folder --> all 3 get built into ./target/[debug|release]/

## to run

### automatic
`python3 runnAll.py` builds, runs all 3 after another and pipes stdin to the stub, while printing all 3 stdouts with diffrent prefix

- stdin is piped to stub resolver
- when exiting stub or by keyboard interrupt, all 3 get terminated

! runAll.py currently builds as debug build, may be unperformant

### manual
1. build with `cargo build`
2. start the three binaries (in seperate shells)
3. enter domain into stub resolver

## structure

one main lib: `shared`

3 binary projects, each for the specific job

### shared

main struct: `DNSPacket`

can be send via `send_dns_packet()`

cen get received and deserialized via `recv_dns_packet()`

## TODO

- Errorcode needs to be set for response
- No record handling
- authorative flag 
- http proxy in stub resolver