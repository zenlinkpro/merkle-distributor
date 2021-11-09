use frame_support::{
    pallet_prelude::*,
    sp_runtime::traits::{
        AccountIdConversion, AtLeast32BitUnsigned, Keccak256, One, Saturating, StaticLookup,
    },
    sp_std::vec::Vec,
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;

use pallet::*;

use scale_info::TypeInfo;
use sp_core::{Hasher, H256};

#[allow(type_alias_bounds)]
type AccountIdOf<T: Config> = <T as frame_system::Config>::AccountId;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct MerkleMetadata<Balance, CurrencyId, AccountId, BoundString> {
    /// The merkle tree root
    pub merkle_root: H256,
    /// Describe usage of the merkle root
    pub description: BoundString,
    /// The distributed currency
    pub distribute_currency: CurrencyId,
    /// The amount of distributed currency
    pub distribute_amount: Balance,
    /// The account holder distributed currency
    pub distribute_holder: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use std::convert::TryInto;

    #[pallet::config]
    pub trait Config: frame_system::Config + TypeInfo {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The balance type
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen;

        /// The currency ID type
        type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + TypeInfo;

        /// Identifier for the class of merkle dispatcher.
        type MerkleDistributorId: Member
            + Parameter
            + Default
            + Copy
            + MaxEncodedLen
            + One
            + Saturating;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The maximum length of a merkel description stored on-chain.
        #[pallet::constant]
        type StringLimit: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn get_merkle_distributor)]
    pub(super) type MerkleDistributorMetadata<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::MerkleDistributorId,
        MerkleMetadata<T::Balance, T::CurrencyId, T::AccountId, BoundedVec<u8, T::StringLimit>>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn merkle_dispatcher_id)]
    pub(crate) type NextMerkleDistributorId<T: Config> =
        StorageValue<_, T::MerkleDistributorId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        Claim(T::MerkleDistributorId, T::AccountId, T::Balance),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Invalid metadata given.
        BadDescription,
        InvalidMerkleDistributorId,
        MerkleVerifyFailed,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a dispatcher
        #[pallet::weight((
        0,
        DispatchClass::Normal,
        Pays::No
        ))]
        pub fn create_merkle_dispatcher(
            origin: OriginFor<T>,
            merkle_root: H256,
            description: Vec<u8>,
            distribute_currency: T::CurrencyId,
            distribute_amount: T::Balance,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let merkle_dispatcher_id = Self::next_merkle_dispatcher_id();
            let distribute_holder: AccountIdOf<T> =
                T::PalletId::get().into_sub_account(merkle_dispatcher_id);

            let description: BoundedVec<u8, T::StringLimit> = description
                .clone()
                .try_into()
                .map_err(|_| Error::<T>::BadDescription)?;

            MerkleDistributorMetadata::<T>::insert(
                merkle_dispatcher_id,
                MerkleMetadata {
                    merkle_root,
                    description,
                    distribute_currency,
                    distribute_amount,
                    distribute_holder,
                },
            );

            Ok(())
        }

        #[pallet::weight((
        0,
        DispatchClass::Normal,
        Pays::No
        ))]
        #[transactional]
        pub fn claim(
            origin: OriginFor<T>,
            merkle_distributor_id: T::MerkleDistributorId,
            index: u32,
            account: <T::Lookup as StaticLookup>::Source,
            amount: T::Balance,
            merkle_proof: Vec<H256>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let account_id = T::Lookup::lookup(account)?;
            let mut index_data = index.encode();
            let mut account_data = account_id.encode();
            let mut balance_data = amount.encode();

            index_data.append(&mut account_data);
            index_data.append(&mut balance_data);

            let node: H256 = Keccak256::hash(&index_data);

            let merkle: MerkleMetadata<
                T::Balance,
                T::CurrencyId,
                T::AccountId,
                BoundedVec<u8, T::StringLimit>,
            > = Self::get_merkle_distributor(merkle_distributor_id)
                .ok_or(Error::<T>::InvalidMerkleDistributorId)?;

            ensure!(
                Self::verify_merkle_proof(&merkle_proof, merkle.merkle_root, node),
                Error::<T>::MerkleVerifyFailed
            );

            Self::deposit_event(Event::<T>::Claim(merkle_distributor_id, account_id, amount));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub(crate) fn next_merkle_dispatcher_id() -> T::MerkleDistributorId {
            let next_merkle_distributor_id = Self::merkle_dispatcher_id();
            NextMerkleDistributorId::<T>::mutate(|current| {
                *current = current.saturating_add(One::one())
            });
            next_merkle_distributor_id
        }

        pub(crate) fn verify_merkle_proof(
            merkle_proof: &[H256],
            merkle_root: H256,
            leaf: H256,
        ) -> bool {
            let mut computed_hash = leaf;

            for i in 0..(merkle_proof.len() - 1) {
                let proof_element = merkle_proof[i];
                if computed_hash <= proof_element {
                    // Hash(current computed hash + current element of the proof)
                    let mut pack = computed_hash.encode();
                    pack.append(&mut proof_element.encode());
                    computed_hash = Keccak256::hash(&pack);
                } else {
                    // Hash(current element of the proof + current computed hash)
                    let mut pack = proof_element.encode();
                    pack.append(&mut computed_hash.encode());
                    computed_hash = Keccak256::hash(&pack);
                }
            }
            computed_hash == merkle_root
        }
    }
}
