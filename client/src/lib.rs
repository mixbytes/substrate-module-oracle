extern crate codec;
extern crate keyring;
extern crate log;
extern crate rand;
extern crate runtime_primitives;
extern crate substrate_api_client;

use hex::FromHexError;

use runtime_primitives::MultiSignature;
use substrate_api_client::utils::{hexstr_to_u64, hexstr_to_vec};
use substrate_api_client::{compose_extrinsic, Api};

use codec::{Decode, Encode};
use primitives::crypto::Pair;

use primitives::H256 as Hash;

use runtime_primitives::AccountId32;
use substrate_api_client::extrinsic::xt_primitives::UncheckedExtrinsicV4;

pub type Id = u64;
pub type ValueType = u128;
pub type ValueName = Vec<u8>;
pub type Moment = u64;

pub const MODULE: &str = "Oracle";
pub const CREATE: &str = "create_oracle";
pub const COMMIT_VALUE: &str = "commit_external_value";
pub const ORACLES: &str = "OraclesMap";
pub const ID_SEQUENCE: &str = "IdSequence";

#[derive(Encode, Decode)]
struct ExternalValueData {
    value: ValueType,
    moment: Moment,
}

#[derive(Encode, Decode)]
pub struct Oracle {
    owner: AccountId32,
    name: ValueName,
    data: Option<ExternalValueData>,
}

pub trait ModuleApi {
    fn create_oracle(&self, external_name: ValueName, value: Option<ValueType>) -> Option<Hash>;
    fn commit_external_value(&self, oracle_id: &Id, external_value: ValueType) -> Option<Hash>;

    fn get_current_value(&self, oracle_id: &Id) -> Option<ValueType>;
    fn get_next_oracle_id(&self) -> Option<Id>;
    fn get_oracle_data(&self, oracle_id: &Id) -> Option<Oracle>;
}

pub type CreateOracleCallXt = UncheckedExtrinsicV4<([u8; 2], Vec<u8>, Option<ValueType>)>;
pub type CommitValueCallXt = UncheckedExtrinsicV4<([u8; 2], Id, ValueType)>;

impl<P: Pair> ModuleApi for Api<P>
where
    P: Pair,
    MultiSignature: From<P::Signature>,
{
    fn create_oracle(&self, external_name: ValueName, value: Option<ValueType>) -> Option<Hash> {
        let extrinsic: CreateOracleCallXt =
            compose_extrinsic!(self, MODULE, CREATE, external_name, value);
        self.send_extrinsic(extrinsic.hex_encode())
            .map_err(|err| log::error!("Send extrinsic error: {}", err))
            .ok()
    }

    fn commit_external_value(&self, oracle_id: &Id, external_value: ValueType) -> Option<Hash> {
        let extrinsic: CommitValueCallXt = compose_extrinsic!(
            self,
            MODULE,
            COMMIT_VALUE,
            oracle_id.clone(),
            external_value
        );
        self.send_extrinsic(extrinsic.hex_encode())
            .map_err(|err| log::error!("Send extrinsic error: {}", err))
            .ok()
    }

    fn get_current_value(&self, oracle_id: &Id) -> Option<ValueType> {
        self.get_oracle_data(&oracle_id)
            .and_then(|oracle| oracle.data)
            .and_then(|data| Some(data.value))
    }

    fn get_next_oracle_id(&self) -> Option<Id> {
        self.get_storage(MODULE, ID_SEQUENCE, None)
            .map_err(|err| log::error!("{}", err))
            .ok()
            .and_then(|res| {
                hexstr_to_u64(res)
                    .map_err(|err| log::error!("{}", err))
                    .ok()
            })
    }

    fn get_oracle_data(&self, oracle_id: &Id) -> Option<Oracle> {
        match self.get_storage(MODULE, ORACLES, Some(oracle_id.to_owned().encode())) {
            Ok(raw) => match hexstr_to_vec(raw) {
                Ok(raw_vec) => Some(Decode::decode(&mut raw_vec.as_slice()).unwrap()),
                Err(err) => {
                    log::error!("{}", err);
                    None
                }
            },
            Err(err) => {
                log::error!("{}", err);
                None
            }
        }
    }
}
