#![allow(unused_imports)]
#![allow(dead_code)]

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager,
     VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap,
     storable::Bound, Storable};
use std::{borrow::Cow, cell::RefCell};
use serde::Deserialize;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Deserialize, Clone)]
struct Listing {
    title: String,
    description: String,
    starting_price: u64,
    sold: bool,
    owner: candid::Principal,
}

#[derive(CandidType, Deserialize, Clone)]
struct CreateListing {
    title: String,
    description: String,
    starting_price: u64,
}

impl Storable for Listing {
    type Id = MemoryId;

    fn id(&self) -> Self::Id {
        MemoryId::from(self.title.as_bytes())
    }

    fn encode(&self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn decode(bytes: &[u8]) -> Self {
        Decode!(&bytes).unwrap()
    }
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static LISTING_MAP: RefCell<StableBTreeMap<u64, Proposal, Memory>> =
        RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(
            |m| m.borrow().get(MemoryId::new(0)))));
}