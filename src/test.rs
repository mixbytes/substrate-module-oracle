use keyring::AccountKeyring;
use log::{debug, error};
use oracle_client::ModuleApi;
use primitives::H256 as Hash;
use substrate_api_client::Api;

use oracle_module_runtime::{oracle, Event};
use std::sync::mpsc::channel;
use substrate_api_client::utils::hexstr_to_vec;

use codec::Decode;
const URL: &str = "127.0.0.1:9944";

macro_rules! init_api {
    ($name:ident) => {
        Api::new(format!("ws://{}", URL)).set_signer(AccountKeyring::$name.pair())
    };
}

#[test]
fn autoincrement() {
    let api = init_api!(Alice);

    let id = api.get_next_oracle_id().expect("Error in get from store");

    debug!("Get id {} from storage", id);

    assert!(api
        .create_oracle("++".to_owned().into_bytes(), None)
        .is_some());

    debug!("Create oracle");
    assert_eq!(
        api.get_next_oracle_id().expect("Get next oracle id"),
        id + 1
    );
}

#[test]
fn create_oracle() {
    let alice_api = init_api!(Alice);

    let name = "test_create_oracle".to_owned().into_bytes();
    assert!(alice_api.create_oracle(name.clone(), None).is_some());
    debug!("Send create oracle via api.");

    let (events_in, events_out) = channel();
    alice_api.subscribe_events(events_in.clone());

    for _ in 1..10 {
        debug!("Start recv events.");
        let raw = events_out.recv().unwrap();
        let event_str = hexstr_to_vec(raw).unwrap();

        match Vec::<system::EventRecord<Event, Hash>>::decode(&mut event_str.as_slice()) {
            Ok(events) => {
                for evr in &events {
                    if let Event::oracle(ev) = &evr.event {
                        if let oracle::RawEvent::OracleCreated(
                            _oracle_id,
                            account_id,
                            external_value_name,
                        ) = &ev
                        {
                            assert_eq!(account_id, &AccountKeyring::Alice.to_account_id());
                            assert_eq!(external_value_name, &name);
                            return;
                        }
                    }
                }
            }
            Err(err) => {
                error!("{}", err);
                assert!(false, "Error in event decoding");
            }
        }
    }
    assert!(false, "Error: Oracle not created.");
}

#[test]
fn commit_value() {
    let api = init_api!(Alice);
    let id = api.get_next_oracle_id().expect("Error in get from store");
    api.create_oracle("commit".to_owned().into_bytes(), None);

    let value = 100u128;
    api.commit_external_value(&id, value);

    let bob_api = init_api!(Bob);
    assert_eq!(
        bob_api
            .get_current_value(&id)
            .expect("Can't get value from store."),
        value
    );
}
