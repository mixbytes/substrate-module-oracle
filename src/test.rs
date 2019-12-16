use keyring::AccountKeyring;
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
fn oracle_commit_and_store() {}

#[test]
fn test_autoincrement()
{
    let api = init_api!(Alice);

    let id = api.get_next_oracle_id().unwrap();
    assert!(api
        .create_oracle("test_autoincrement".to_owned().into_bytes(), None)
        .is_some());
    assert_ne!(api.get_next_oracle_id().unwrap(), id);
}

#[test]
fn test_create_oracle()
{
    let alice_api = init_api!(Alice);

    let name = "test_create_oracle".to_owned().into_bytes();
    assert!(alice_api.create_oracle(name.clone(), None).is_some());

    let (events_in, events_out) = channel();
    alice_api.subscribe_events(events_in.clone());

    for _ in 1..10
    {
        let raw = events_out.recv().unwrap();
        let event_str = hexstr_to_vec(raw).unwrap();

        match Vec::<system::EventRecord<Event, Hash>>::decode(&mut event_str.as_slice())
        {
            Ok(events) =>
            {
                for evr in &events
                {
                    if let Event::oracle(ev) = &evr.event
                    {
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
            Err(err) =>
            {
                log::error!("{}", err);
                assert!(false, "Error in event decoding");
            }
        }
    }
    assert!(false, "Error: Oracle not created.");
}
