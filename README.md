# Substrate Simple Oracle

Simple oracle substrate-module written in Rust.

## Build

### Install dependencies

Run script `./ci/install_rust.sh`.

### Build

```bash
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

### Example 

#### External rust owner

```rust
use keyring::AccountKeyring;
use substrate_api_client::Api;
use oracle_client::{ModuleApi,ValueType};

fn main()
{
    let api = Api::new(format!("ws://{}", URL)).set_signer(AccountKeyring::Alice);
    let id = api.get_next_oracle_id().expect("Error in get from store");
    api.create_oracle("oracle_name".to_owned().into_bytes(), None).unwrap("Error in create oracle");
    let external_value = ValueType::from(100);
    api.commit_external_value(&id, external_value);
}
```

#### External rust client

```rust
use keyring::AccountKeyring;
use substrate_api_client::Api;
use oracle_client::{ModuleApi,ValueType};

fn main()
{
    let api = Api::new(format!("ws://{}", URL)).set_signer(AccountKeyring::Bob);
    let id: u64 = 1; // getting id of a particular oracle can happen off-chain
    println!("{}", api.get_current_value(&id).expect("Can't get value from store"));
}
```

#### Example SRML module
```rust
pub trait Trait: oracle::Trait {...}

// In `decl_module` part
fn func(origin) {
    let oracle_id = ...;
    let data: OracleData = OraclesMap::<T>::get(oracle_id)?;
    if let Some((external_value, timestamp)) = data.external_value {
        // Here we have `external_value`
    }
}

```
