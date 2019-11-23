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
    type ExAssetValueType: Member + Parameter + SimpleArithmetic + Default + Copy;
    type OracleId: Parameter + SimpleArithmetic + Default + Copy;
}

#[derive(Encode, Decode)]
struct OracleData<T: Trait>
{
    source_node: <T as system::Trait>::AccountId,
    asset_name: AssetName,
    asset_value: Option<(
        <T as Trait>::ExAssetValueType,
        <T as timestamp::Trait>::Moment,
    )>,
}

impl<T: Trait> Default for OracleData<T>
{
    fn default() -> Self
    {
        OracleData {
            source_node: <T as system::Trait>::AccountId::default(),
            asset_name: AssetName::default(),
            asset_value: None,
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

        pub fn commit_asset_value(origin, oracle_id: T::OracleId, new_value: T::ExAssetValueType) -> Result
        {
            let who = ensure_signed(origin)?;

            if <OraclesMap<T>>::get(oracle_id).source_node != who
            {
                Err("Can't commit price: no permission")
            }
            else
            {
                <OraclesMap<T>>::mutate(oracle_id, |data| {
                    let now = <timestamp::Module<T>>::get();
                    data.asset_value = Some((new_value, now));
                });
                Self::deposit_event(RawEvent::CourseStored(oracle_id));
                Ok(())
            }
        }

        pub fn create_oracle(origin, asset_name: Vec<u8>)
        {
            let who: T::AccountId = ensure_signed(origin)?;

            <OraclesMap<T>>::insert(Self::get_next_oracle_id(),
                OracleData {
                    source_node: who,
                    asset_name: asset_name,
                    asset_value: None
                });
        }
   }
}

decl_event!(
    pub enum Event<T>
    where
        OracleId = <T as Trait>::OracleId,
    {
        CourseStored(OracleId),
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
        if Self::last_oracle_id() != T::OracleId::default()
        {
            Some(Self::last_oracle_id())
        }
        else
        {
            None
        }
    }

    pub fn get_current_asset_value(oracle_id: T::OracleId) -> Option<T::ExAssetValueType>
    {
        match <OraclesMap<T>>::get(oracle_id).asset_value
        {
            Some((val, _time)) => Some(val),
            None => None,
        }
    }
}
