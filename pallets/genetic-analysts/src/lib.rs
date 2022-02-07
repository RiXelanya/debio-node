#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
pub use pallet::*;
pub use scale_info::TypeInfo;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod interface;
pub use crate::interface::GeneticAnalystInterface;
use frame_support::pallet_prelude::*;
use traits_genetic_analyst_qualifications::GeneticAnalystQualificationOwnerInfo;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub enum StakeStatus {
	Staked,
	WaitingForStaked,
	Unstaked,
}
impl Default for StakeStatus {
	fn default() -> Self {
		StakeStatus::Unstaked
	}
}

// GeneticAnalystInfo Struct
// Used as parameter of dispatchable calls
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct GeneticAnalystInfo<Moment> {
	pub first_name: Vec<u8>,
	pub last_name: Vec<u8>,
	pub gender: Vec<u8>,
	pub date_of_birth: Moment,
	pub email: Vec<u8>,
	pub phone_number: Vec<u8>,
	pub specialization: Vec<u8>,
	pub stake_amount: i64,
	pub stake_status: StakeStatus,
}

// GeneticAnalyst Struct
// the fields (excluding account_id and qualifications) come from GeneticAnalystInfo struct
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct GeneticAnalyst<AccountId, Hash, Moment>
where
	Hash: PartialEq + Eq,
{
	pub account_id: AccountId,
	pub qualifications: Vec<Hash>,
	pub info: GeneticAnalystInfo<Moment>,
}

impl<AccountId, Hash, Moment> GeneticAnalyst<AccountId, Hash, Moment>
where
	Hash: PartialEq + Eq,
{
	pub fn new(account_id: AccountId, info: GeneticAnalystInfo<Moment>) -> Self {
		Self { account_id, qualifications: Vec::<Hash>::new(), info }
	}

	fn update_info(&mut self, info: GeneticAnalystInfo<Moment>) {
		self.info = info;
	}

	pub fn get_account_id(&self) -> &AccountId {
		&self.account_id
	}

	pub fn add_qualification(&mut self, qualification_id: Hash) {
		self.qualifications.push(qualification_id);
	}

	pub fn remove_qualification(&mut self, qualification_id: Hash) {
		if let Some(pos) = &self.qualifications.iter().position(|x| *x == qualification_id) {
			self.qualifications.remove(*pos);
		}
	}
}

impl<T, AccountId, Hash, Moment> GeneticAnalystQualificationOwnerInfo<T> for GeneticAnalyst<AccountId, Hash, Moment>
where
	Hash: PartialEq + Eq,
	T: frame_system::Config<AccountId = AccountId>,
{
	fn get_owner_id(&self) -> &AccountId {
		self.get_account_id()
	}
}

#[frame_support::pallet]
pub mod pallet {
	use crate::{interface::GeneticAnalystInterface, GeneticAnalyst, GeneticAnalystInfo, *};
	use codec::EncodeLike;
	use frame_support::{dispatch::DispatchResultWithPostInfo, traits::Currency};
	use frame_system::pallet_prelude::*;
	pub use sp_std::prelude::*;
	pub use traits_genetic_analyst_qualifications::{
		GeneticAnalystQualificationOwner, GeneticAnalystQualificationsProvider,
	};
	use traits_user_profile::UserProfileProvider;

	#[pallet::config]
	/// Configure the pallet by specifying the parameters and types on which it depends.
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type GeneticAnalystQualifications: GeneticAnalystQualificationsProvider<Self>;
		type EthereumAddress: Clone
			+ Copy
			+ PartialEq
			+ Eq
			+ Encode
			+ EncodeLike
			+ Decode
			+ Default
			+ TypeInfo
			+ sp_std::fmt::Debug;
		type UserProfile: UserProfileProvider<Self, Self::EthereumAddress>;
		type GeneticAnalystWeightInfo: WeightInfo;
	}

	// ----- This is template code, every pallet needs this ---
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
	// --------------------------------------------------------

	// ---- Types ----------------------
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type HashOf<T> = <T as frame_system::Config>::Hash;
    pub type MomentOf<T> = <T as pallet_timestamp::Config>::Moment;
	pub type GeneticAnalystOf<T> = GeneticAnalyst<AccountIdOf<T>, HashOf<T>, MomentOf<T>>;

	// ----- Storage ------------------
	/// Get GeneticAnalyst by account id
	/// AccountId => GeneticAnalyst
	#[pallet::storage]
	#[pallet::getter(fn genetic_analyst_by_account_id)]
	pub type GeneticAnalysts<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, GeneticAnalystOf<T>>;

	/// Get total genetic_analyst count
	/// u32
	#[pallet::storage]
	#[pallet::getter(fn genetic_analyst_count)]
	pub type GeneticAnalystCount<T> = StorageValue<_, u64>;
	// -----------------------------------

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// User AccountId registered as genetic_analyst
		/// parameters. [GeneticAnalyst, who]
		GeneticAnalystRegistered(GeneticAnalystOf<T>, AccountIdOf<T>),
		/// GeneticAnalyst information updated
		/// parameters. [GeneticAnalyst, who]
		GeneticAnalystUpdated(GeneticAnalystOf<T>, AccountIdOf<T>),
		/// GeneticAnalyst deleted
		/// parameters. [GeneticAnalyst, who]
		GeneticAnalystDeleted(GeneticAnalystOf<T>, AccountIdOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Account already has genetic_analyst registered
		GeneticAnalystAlreadyRegistered,
		/// GeneticAnalyst identified by the AccountId does not exist
		GeneticAnalystDoesNotExist,
		/// GeneticAnalyst is not the owner of the qualification
		GeneticAnalystIsNotOwner,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::GeneticAnalystWeightInfo::register_genetic_analyst())]
		pub fn register_genetic_analyst(
			origin: OriginFor<T>,
			genetic_analyst_info: GeneticAnalystInfo<MomentOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			match Self::create_genetic_analyst(&who, &genetic_analyst_info) {
				Ok(genetic_analyst) => {
					Self::deposit_event(Event::GeneticAnalystRegistered(genetic_analyst, who.clone()));
					Ok(().into())
				},
				Err(error) => Err(error.into()),
			}
		}

		#[pallet::weight(T::GeneticAnalystWeightInfo::update_genetic_analyst())]
		pub fn update_genetic_analyst(
			origin: OriginFor<T>,
			genetic_analyst_info: GeneticAnalystInfo<MomentOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			match <Self as GeneticAnalystInterface<T>>::update_genetic_analyst(&who, &genetic_analyst_info) {
				Ok(genetic_analyst) => {
					Self::deposit_event(Event::GeneticAnalystUpdated(genetic_analyst, who.clone()));
					Ok(().into())
				},
				Err(error) => Err(error.into()),
			}
		}

		#[pallet::weight(T::GeneticAnalystWeightInfo::deregister_genetic_analyst())]
		pub fn deregister_genetic_analyst(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// Check if user is a genetic_analyst
			let genetic_analyst = Self::genetic_analyst_by_account_id(&who);
			if genetic_analyst == None {
				return Err(Error::<T>::GeneticAnalystDoesNotExist.into())
			}

			match <Self as GeneticAnalystInterface<T>>::delete_genetic_analyst(&who) {
				Ok(genetic_analyst) => {
					Self::deposit_event(Event::GeneticAnalystDeleted(genetic_analyst, who.clone()));
					Ok(().into())
				},
				Err(error) => Err(error.into()),
			}
		}
	}
}

impl<T: Config> GeneticAnalystInterface<T> for Pallet<T> {
	type Error = Error<T>;
	type GeneticAnalystInfo = GeneticAnalystInfo<MomentOf<T>>;
	type GeneticAnalyst = GeneticAnalystOf<T>;

	fn create_genetic_analyst(
		account_id: &T::AccountId,
		genetic_analyst_info: &Self::GeneticAnalystInfo,
	) -> Result<Self::GeneticAnalyst, Self::Error> {
		if GeneticAnalysts::<T>::contains_key(account_id) {
			return Err(Error::<T>::GeneticAnalystAlreadyRegistered)
		}
		let genetic_analyst = GeneticAnalyst::new(account_id.clone(), genetic_analyst_info.clone());
		// Insert to Storage
		GeneticAnalysts::<T>::insert(account_id, &genetic_analyst);

		// Increment Count
		Self::add_genetic_analyst_count();

		Ok(genetic_analyst)
	}

	fn update_genetic_analyst(
		account_id: &T::AccountId,
		genetic_analyst_info: &Self::GeneticAnalystInfo,
	) -> Result<Self::GeneticAnalyst, Self::Error> {
		let genetic_analyst = GeneticAnalysts::<T>::get(account_id);
		if genetic_analyst == None {
			return Err(Error::<T>::GeneticAnalystDoesNotExist)
		}
		let mut genetic_analyst = genetic_analyst.unwrap();

		genetic_analyst.update_info(genetic_analyst_info.clone());

		GeneticAnalysts::<T>::insert(account_id, &genetic_analyst);

		Ok(genetic_analyst)
	}

	fn delete_genetic_analyst(account_id: &T::AccountId) -> Result<Self::GeneticAnalyst, Self::Error> {
		let genetic_analyst = GeneticAnalysts::<T>::get(account_id);
		if genetic_analyst == None {
			return Err(Error::<T>::GeneticAnalystDoesNotExist)
		}
		let genetic_analyst = genetic_analyst.unwrap();
		// Delete genetic_analyst's qualifications
		for qualification_id in &genetic_analyst.qualifications {
			let _result =
				T::GeneticAnalystQualifications::delete_qualification(account_id, qualification_id);
		}
		GeneticAnalysts::<T>::remove(&genetic_analyst.account_id);
		Self::sub_genetic_analyst_count();

		Ok(genetic_analyst)
	}

	fn genetic_analyst_by_account_id(account_id: &T::AccountId) -> Option<Self::GeneticAnalyst> {
		Self::genetic_analyst_by_account_id(account_id)
	}
}

impl<T: Config> Pallet<T> {
	// Add genetic_analyst count
	pub fn add_genetic_analyst_count() {
		let genetic_analyst_count = <GeneticAnalystCount<T>>::get().unwrap_or(0);
		<GeneticAnalystCount<T>>::put(genetic_analyst_count.wrapping_add(1));
	}

	// Subtract genetic_analyst count
	pub fn sub_genetic_analyst_count() {
		let genetic_analyst_count = <GeneticAnalystCount<T>>::get().unwrap_or(1);
		GeneticAnalystCount::<T>::put(genetic_analyst_count - 1);
	}
}

impl<T: Config> GeneticAnalystQualificationOwner<T> for Pallet<T> {
	type Owner = GeneticAnalyst<T::AccountId, T::Hash, MomentOf<T>>;

	/// User can create qualification if he/she is a genetic_analyst
	fn can_create_qualification(user_id: &T::AccountId) -> bool {
		GeneticAnalysts::<T>::contains_key(user_id)
	}

	fn get_owner(id: &T::AccountId) -> Option<Self::Owner> {
		GeneticAnalysts::<T>::get(id)
	}

	fn associate(owner_id: &T::AccountId, qualification_id: &T::Hash) {
		<GeneticAnalysts<T>>::mutate(owner_id, |genetic_analyst| {
			match genetic_analyst {
				None => (), // If genetic_analyst does not exist, do nothing
				Some(genetic_analyst) => {
					genetic_analyst.add_qualification(*qualification_id);
				},
			}
		});
	}

	fn disassociate(owner_id: &T::AccountId, qualification_id: &T::Hash) {
		GeneticAnalysts::<T>::mutate(owner_id, |genetic_analyst| match genetic_analyst {
			None => (),
			Some(genetic_analyst) => {
				genetic_analyst.remove_qualification(*qualification_id);
			},
		});
	}
}