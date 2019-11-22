use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, storage::StorageMap, Parameter,
    StorageValue,
};

use support::codec::{Decode, Encode};

use rstd::vec::Vec;
use runtime_primitives::traits::{Member, One, SimpleArithmetic};
use system::ensure_signed;

pub trait Trait: system::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type ExAssetValueType: Member + Parameter + SimpleArithmetic + Default + Copy;
    type OracleId: Parameter + SimpleArithmetic + Default + Copy;
}

#[derive(Encode, Decode, Default)]
struct OracleData<AccountId, AssetName, AssetValueType>
{
    source: AccountId,
    asset_name: AssetName,
    asset_value: Option<AssetValueType>,
}

decl_storage! {
    trait Store for Module<T: Trait> as Oracle
    {
        NextOracleId get(next_oracle_id): T::OracleId;
        OraclesMap: map T::OracleId => OracleData<T::AccountId, Vec<u8>, T::ExAssetValueType>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin
    {
        fn deposit_event<T>() = default;

        pub fn commit_asset_value(origin, oracle_id: T::OracleId, new_value: T::ExAssetValueType) -> Result
        {
            let who = ensure_signed(origin)?;

            if <OraclesMap<T>>::get(oracle_id).source != who
            {
                Err("Can't commit price: no permission")
            }
            else
            {
                <OraclesMap<T>>::mutate(oracle_id, |data| {data.asset_value = Some(new_value);});
                Self::deposit_event(RawEvent::CourseStored(oracle_id));
                Ok(())
            }
        }

        pub fn create_oracle(origin, asset_name: Vec<u8>)
        {
            let who: T::AccountId = ensure_signed(origin)?;

            <OraclesMap<T>>::insert(Self::get_next_oracle_id(),
                OracleData {
                 source: who,
                 asset_name: asset_name,
                 asset_value: None});
        }
   }
}

decl_event!(
    pub enum Event<T>
    where OracleId = <T as Trait>::OracleId,
    {
        CourseStored(OracleId),
    }
);

impl<T: Trait> Module<T>
{
    fn get_next_oracle_id() -> T::OracleId
    {
        let id: T::OracleId = Self::next_oracle_id();
        <NextOracleId<T>>::mutate(|id| *id += One::one());
        id
    }

    pub fn get_max_oracle_id() -> T::OracleId
    {
        Self::next_oracle_id() - One::one()
    }

    pub fn get_current_asset_value(oracle_id: T::OracleId) -> Option<T::ExAssetValueType>
    {
        <OraclesMap<T>>::get(oracle_id).asset_value.clone()
    }
}
