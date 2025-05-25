#![allow(unused_imports)]
#![allow(dead_code)]

use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable, storable::Bound};
use std::cell::RefCell;
use serde::Deserialize;
use std::borrow::Cow;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Deserialize, Clone)]
struct Listing {
    title: String,
    description: String,
    starting_price: u64,
    sold: bool,
    owner: Principal,
}

#[derive(CandidType, Deserialize, Clone)]
struct CreateListing {
    title: String,
    description: String,
    starting_price: u64,
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
