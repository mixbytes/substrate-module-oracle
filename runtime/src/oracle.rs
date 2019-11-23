use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, storage::StorageMap, Parameter,
    StorageValue,
};

use support::codec::{Decode, Encode};
use timestamp;

use rstd::vec::Vec;
use runtime_primitives::traits::{Member, One, SimpleArithmetic};
use system::ensure_signed;

type AssetName = Vec<u8>;

pub trait Trait: timestamp::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type ExternalValueType: Member + Parameter + SimpleArithmetic + Default + Copy;
    type OracleId: Parameter + SimpleArithmetic + Default + Copy;
}

#[derive(Encode, Decode)]
struct OracleData<T: Trait>
{
    source_node: <T as system::Trait>::AccountId,
    external_name: AssetName,
    external_value: Option<(
        <T as Trait>::ExternalValueType,
        <T as timestamp::Trait>::Moment,
    )>,
}

impl<T: Trait> Default for OracleData<T>
{
    fn default() -> Self
    {
        OracleData {
            source_node: <T as system::Trait>::AccountId::default(),
            external_name: AssetName::default(),
            external_value: None,
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Oracle
    {
        NextOracleId get(last_oracle_id): T::OracleId;
        OraclesMap: map T::OracleId => OracleData<T>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin
    {
        fn deposit_event<T>() = default;

        pub fn commit_asset_value(origin, oracle_id: T::OracleId, new_asset_value: T::ExternalValueType) -> Result
        {
            let who = ensure_signed(origin)?;

            if <OraclesMap<T>>::get(oracle_id).source_node != who
            {
                Err("Can't commit price: no permission")
            }
            else
            {
                <OraclesMap<T>>::mutate(oracle_id, |data| {
                    data.external_value = Some((new_asset_value, <timestamp::Module<T>>::get()));
                });
                Self::deposit_event(RawEvent::CourseStored(oracle_id, new_asset_value));
                Ok(())
            }
        }

        pub fn create_oracle(origin, external_name: Vec<u8>)
        {
            let who: T::AccountId = ensure_signed(origin)?;

            <OraclesMap<T>>::insert(Self::get_next_oracle_id(),
                OracleData {
                    source_node: who,
                    external_name: external_name,
                    external_value: None
                });
        }
   }
}

decl_event!(
    pub enum Event<T>
    where OracleId = <T as Trait>::OracleId,
          ExternalValueType = <T as Trait>::ExternalValueType,
    {
        CourseStored(OracleId, ExternalValueType),
    }
);

impl<T: Trait> Module<T>
{
    fn get_next_oracle_id() -> T::OracleId
    {
        let id: T::OracleId = Self::last_oracle_id();
        <NextOracleId<T>>::mutate(|id| *id += One::one());
        id
    }

    pub fn get_max_oracle_id() -> Option<T::OracleId>
    {
        if Self::last_oracle_id() != T::OracleId::default() { Some(Self::last_oracle_id()) } else { None }
    }

    pub fn get_current_asset_value(oracle_id: T::OracleId) -> Option<T::ExternalValueType>
    {
        match <OraclesMap<T>>::get(oracle_id).external_value
        {
            Some((val, _time)) => Some(val),
            None => None,
        }
    }
}
