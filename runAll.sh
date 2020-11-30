echo "Starting all, terminal pipe to stub"

if cargo build; then

    ./target/debug/dns_server
    ./target/debug/recursive_resolver
    ./target/debug/stub_resolver

else
    echo "build failed"
fi
