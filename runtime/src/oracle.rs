use support::{decl_event, decl_module, decl_storage, dispatch::Result, StorageValue};
use system::ensure_signed;
use rstd::vec::Vec;

type AssetType = u128;

pub trait Trait: system::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage!
{
    trait Store for Module<T: Trait> as Oracle
    {
        pub AssetValue get(asset_value): Option<AssetType>;
        pub AssetName get(asset_name) config(): Vec<u8>;
        OracleId get(oracle_id) config(): T::AccountId;
    }
}

decl_module! 
{
    pub struct Module<T: Trait> for enum Call where origin: T::Origin 
    {
        fn deposit_event<T>() = default;

        pub fn store_course(origin, course: AssetType) -> Result 
        {
            let who = ensure_signed(origin)?;

            if <OracleId<T>>::get() != who 
            {
                Err("Can't send price: no permission")
            }
            else
            {
                <AssetValue<T>>::put(course);
                Self::deposit_event(RawEvent::CourseStored(who, course));
                Ok(())
            }
        }
    }

}

decl_event!(
    pub enum Event<T>
    where AccountId = <T as system::Trait>::AccountId,
    {
        CourseStored(AccountId, AssetType),
    }
);
