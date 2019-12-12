use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, storage::StorageMap, Parameter,
    StorageValue,
};

use codec::{Decode, Encode};
use rstd::result;
use rstd::vec::Vec;
use sr_primitives::traits::{CheckedAdd, Member, One, SimpleArithmetic};
use system::ensure_signed;

type ExternalValueName = Vec<u8>;

#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct OracleData<T: Trait> {
    /// Account with permission to commit external value into storage
    source_account: <T as system::Trait>::AccountId,

    /// It's name of external value
    external_name: ExternalValueName,

    /// External value and external value time of store
    external_value: Option<(
        <T as Trait>::ExternalValueType,
        <T as timestamp::Trait>::Moment,
    )>,
}

/// Default trait is needed for use type in storage map container
impl<T: Trait> Default for OracleData<T> {
    fn default() -> Self {
        OracleData {
            source_account: <T as system::Trait>::AccountId::default(),
            external_name: ExternalValueName::default(),
            external_value: None,
        }
    }
}

/// Current module config types
pub trait Trait: timestamp::Trait {
    /// Substrate needed
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type ExternalValueType: Member + Parameter + SimpleArithmetic + Default + Copy;
    type OracleId: Parameter + Member + SimpleArithmetic + Default + Copy;
}

decl_storage! {
    trait Store for Module<T: Trait> as Oracle {
        /// Oracles id sequence
        /// Start with len of default oracles
        IdSequence
            get(last_oracle_id)
            build(|conf: &GenesisConfig<T>| {
            let mut res = T::OracleId::default();
            for _ in 0..conf.default_oracles.len() {
                res = res.checked_add(&One::one()).expect("Too much oracles in config, T::OracleId overflow.");
            }
            res
        }): T::OracleId;

        /// Oracles
        /// Can be initialized from GenesisConfig
        pub OraclesMap
            get(oracles)
            build(|conf: &GenesisConfig<T>| {
            let mut oracle_id = T::OracleId::default();

            let post_inc = |id: &mut T::OracleId| {
                let tmp = id.clone();
                id.checked_add(&One::one()).expect("Too much oracles in config, T::OracleId overflow.");
                tmp
            };

            conf.default_oracles.iter().map(|&(ref source_account, ref external_name)| {
                (post_inc(&mut oracle_id),
                 OracleData { source_account: source_account.clone(),
                              external_name: external_name.clone(),
                              external_value: None
                 })
            }).collect::<Vec<_>>()
        }): map T::OracleId => OracleData<T>;
    }

    // Add custom field to module configuration
    add_extra_genesis {
        config(default_oracles): Vec<(T::AccountId, ExternalValueName)>;
    }
}

// External API. Can be called from external client.
decl_module! {
 pub struct Module<T: Trait> for enum Call where origin: T::Origin {
     fn deposit_event() = default;

     pub fn commit_external_value(origin, oracle_id: T::OracleId, new_external_value: T::ExternalValueType) -> Result {
         let who = ensure_signed(origin)?;

         if OraclesMap::<T>::get(oracle_id).source_account != who {
             Err("Can't commit external value: no permission")
         } else {
             OraclesMap::<T>::mutate(oracle_id, |data| {
                 data.external_value = Some((new_external_value, timestamp::Module::<T>::get()));
             });
             Self::deposit_event(RawEvent::ExternalValueStored(oracle_id, new_external_value));
             Ok(())
         }
     }

     pub fn create_oracle(origin, external_name: Vec<u8>, start_external_value: Option<T::ExternalValueType>) {
         let who: T::AccountId = ensure_signed(origin)?;

         OraclesMap::<T>::insert(Self::get_next_oracle_id()?,
             OracleData {
                 source_account: who,
                 external_name: external_name,
                 external_value: match start_external_value {
                     Some(ex_value) => Some((ex_value, <timestamp::Module<T>>::get())),
                     None => None,
                 },
             });
     }
} }

decl_event!(
    pub enum Event<T>
    where OracleId = <T as Trait>::OracleId, ExternalValueType = <T as Trait>::ExternalValueType {
        ExternalValueStored(OracleId, ExternalValueType),
    }
);

// Internal API. Can be called from other modules
impl<T: Trait> Module<T> {
    fn get_next_oracle_id() -> result::Result<T::OracleId, &'static str> {
        let mut result = Ok(Self::last_oracle_id());

        IdSequence::<T>::mutate(|id| match id.checked_add(&One::one()) {
            Some(res) => *id = res,
            None => result = Err("T::OracleId overflow. Can't get next id."),
        });

        result
    }

    pub fn get_max_oracle_id() -> Option<T::OracleId> {
        if Self::last_oracle_id() != T::OracleId::default() {
            Some(Self::last_oracle_id() - One::one())
        } else {
            None
        }
    }

    pub fn get_current_asset_value(oracle_id: T::OracleId) -> Option<T::ExternalValueType> {
        match OraclesMap::<T>::get(oracle_id).external_value {
            Some((val, _time)) => Some(val),
            None => None,
        }
    }
}
