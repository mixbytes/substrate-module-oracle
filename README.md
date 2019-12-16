# Substrate Simple Oracle

Simple oracle substrate-module written in Rust.

## Build

### Install dependencies

Run script `./ci/install_rust.sh`.

### Build

```
# Simple build
cargo build

# Run node with oracle
cargo run -- --dev --rpc-external --rpc-port 9955 --ws-external --ws-port 9944 -d ./tmp

# Run unit and CI tests. 
# Before that exec previous command for start node in in the background (**todo**: fix this temporary solution)
cargo test 

```

### How it works

We have module with oracles map and two public methods: `create_oracle` and `commit_external_value`. Any account in substrate network can create own oracle and exclusively commit in it. Any account in network can get this value from module storage. 

Example of external usage by module 'client': **todo**: currently under development.


