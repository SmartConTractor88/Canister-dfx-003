#![allow(unused_imports)]
#![allow(dead_code)]

use std::result::Result;
use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable, storable::Bound};
use std::cell::RefCell;
use serde::Deserialize;
use std::borrow::Cow;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType)]
enum Error {
    AccessRejected,
    ListingNotFound,
    MinimalPriceNotMet,
    ListingAlreadySold,
    UpdateError,
}

#[derive(CandidType, Deserialize, Clone)]
struct Listing {
    title: String,
    description: String,
    starting_price: u64,
    current_price: u64,
    sold: bool,
    owner: Principal,
}

#[derive(CandidType, Deserialize, Clone)]
struct CreateListing {
    title: String,
    description: String,
    starting_price: u64,
}

#[derive(CandidType, Deserialize, Clone)]
struct EditListing {
    title: String,
    description: String,
    sold: bool,
}

impl Storable for Listing {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::Decode!(bytes.as_ref(), Listing).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static LISTING_MAP: RefCell<StableBTreeMap<u64, Listing, Memory>> =
        RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(
            |m| m.borrow().get(MemoryId::new(0)))));

    static LISTING_COUNTER: RefCell<u64> = RefCell::new(0);
}

#[ic_cdk::query]
fn get_listing(id: u64) -> Option<Listing> {
    let id = id - 1; // Adjusting for 0-based index
    LISTING_MAP.with(|p| p.borrow().get(&id))
}

#[ic_cdk::query]
fn get_listing_count() -> u64 {
    LISTING_MAP.with(|p| p.borrow().len() as u64)
}

#[ic_cdk::update]
fn create_listing(input: CreateListing) -> Listing {
    let listing = Listing {
        title: input.title,
        description: input.description,
        starting_price: input.starting_price,
        current_price: input.starting_price,
        sold: false,
        owner: ic_cdk::caller(),
    };

    LISTING_COUNTER.with(|counter| {
        let mut id = counter.borrow_mut();
        let key = *id;
        *id += 1;

        LISTING_MAP.with(|p| {
            p.borrow_mut().insert(key, listing.clone());
        });

        listing
    })
}

#[ic_cdk::update]
fn edit_listing(key: u64, listing: EditListing) -> Result<(), Error> {
    LISTING_MAP.with(|p| {
        let key = key - 1; // Adjusting for 0-based index
        let old_listing_opt = p.borrow().get(&key);
        let old_listing: Listing;

        match old_listing_opt {
            Some(value) => old_listing = value,
            None => return Err(Error::ListingNotFound),
        }

        if ic_cdk::caller() != old_listing.owner {
            return Err(Error::AccessRejected);
        }

        let value = Listing {
            title: listing.title,
            description: listing.description,
            starting_price: old_listing.starting_price,
            current_price: old_listing.current_price,
            sold: listing.sold,
            owner: old_listing.owner,
        };

        let result = p.borrow_mut().insert(key, value);

        match result {
            Some(_) => Ok(()),
            None => Err(Error::UpdateError),
        }
    })
}

#[ic_cdk::update]
fn bid(key: u64, price: u64) -> Result<(), Error> {
    let adjusted_key = key - 1;

    LISTING_MAP.with(|p| {
        let mut map = p.borrow_mut();
        match map.get(&adjusted_key) {
            Some(mut listing) => {
                if listing.sold {
                    return Err(Error::ListingAlreadySold);
                }
                if price <= listing.current_price {
                    return Err(Error::MinimalPriceNotMet);
                }

                listing.current_price = price;
                map.insert(adjusted_key, listing);
                Ok(())
            }
            None => Err(Error::ListingNotFound),
        }
    })
}