# telematik2020_dns

Needed:
- Cargo / rust (preferrably via rustup)
- python (optional) for auto run

## to build all projects
`cargo build` in this folder --> all 3 get built into ./target/[debug|release]/

## to run
`python3 runnAll.py` builds, runs all 3 after another and pipes stdin to the stub, while printing all 3 stdouts with diffrent prefix

## Conventions

prints start with a prefix to identify from which binary they came
- [S] = stub
- [R] = recursive
- [D] = DNS server