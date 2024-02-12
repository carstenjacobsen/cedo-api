use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{Storable, storable::Bound};
use std::{borrow::Cow};

//mod types;
use serde::{Serialize, Deserialize};
use serde_json::json;

use ic_stable_structures::{
    DefaultMemoryImpl, 
    StableBTreeMap,
    memory_manager::MemoryId,
    memory_manager::MemoryManager,
    memory_manager::VirtualMemory,
};

use std::cell::{Cell, RefCell};

use std::collections::HashMap;

const MAX_VALUE_SIZE: u32 = 512;

#[derive(CandidType, Deserialize, Serialize)]
pub struct Order {
    pub order_id: String,
    pub order_type: String,
    pub vendor_id: String,
    pub vendor_order_id: String,
    pub order_amount: f32,
    pub order_amount_currency: String,
    pub tag_id: String,
}

#[derive(CandidType, Deserialize)]
pub struct Orders {
    pub orders: Vec<Order>,
}

#[derive(CandidType, Deserialize)]
pub struct Vendor {
    pub vendor_id: String,
    pub name: String,
    pub logo: String,
    pub tags: Vec<String>,
    pub loyalty: Vec<Loyalty>,
    pub coupon: Vec<Coupon>,
    pub ticket: Vec<Ticket>,
}

#[derive(CandidType, Deserialize)]
pub struct Vendors {
    pub vendors: Vec<Vendor>,
}

#[derive(CandidType, Deserialize)]
pub struct Loyalty {
    pub id: String,
}

#[derive(CandidType, Deserialize)]
pub struct Coupon {
    pub id: String,
}

#[derive(CandidType, Deserialize)]
pub struct Ticket {
    pub id: String,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct VendorMetadata {
    pub vendor_id: String,
    pub name: String,
    pub logo: String,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct Deal {
    pub deal_id: String,
    pub deal_type: String,
    pub headline: String,
    pub description: String,
    pub threshold: f32,
    pub discount: f32,
    pub discount_type: String,
    pub image: String,
    pub vendor_metadata: VendorMetadata,
}

#[derive(CandidType, Deserialize)]
pub struct User {
    pub user_id: String,
    pub deals: Vec<String>,
}

impl Storable for Order {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

impl Storable for Vendor {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

impl Storable for Deal {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEM_MAN_ORDERS: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static ORDERS: RefCell<StableBTreeMap<String, Order, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEM_MAN_ORDERS.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static MEM_MAN_VENDORS: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static VENDORS: RefCell<StableBTreeMap<String, Vendor, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEM_MAN_VENDORS.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );

    static MEM_MAN_DEALS: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static DEALS: RefCell<StableBTreeMap<String, Deal, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEM_MAN_DEALS.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

    static MEM_MAN_USERS: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static USERS: RefCell<StableBTreeMap<String, User, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEM_MAN_USERS.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );
}

#[update]
pub fn create_order(new_order: Order) -> () {
    let tag_id: String = "04864A9ABB2A81".to_string();
    let res = ORDERS.with(|p| p.borrow_mut().insert(new_order.order_id.to_string(), new_order));
}

#[query]
pub fn get_orders() -> Vec<Order> {
    let mut orders: Vec<Order> = Vec::new();

    ORDERS.with(|p| {
        for (k, v) in p.borrow().iter() {
            let ordr = Order {
                order_id: v.order_id,
                order_type: v.order_type,
                vendor_id: v.vendor_id,
                vendor_order_id: v.vendor_order_id,
                order_amount: v.order_amount,
                order_amount_currency: v.order_amount_currency,
                tag_id: k,
            };

            orders.push(ordr);
        }
    }); 

    orders
}

#[query]
pub fn get_order(tag_id: String) -> Option<Order> {
    ORDERS.with(|p| p.borrow().get(&tag_id))
}

#[update]
pub fn complete_order(tag_id: String) -> String{
    ORDERS.with(|p| {p.borrow_mut().remove(&tag_id)});

    tag_id
}

#[update]
pub fn create_vendor(new_vendor: Vendor) -> () {
    let res = VENDORS.with(|p| p.borrow_mut().insert(new_vendor.vendor_id.to_string(), new_vendor));
}

#[query]
pub fn get_vendor(vendor_id: String) -> Option<Vendor> {
    VENDORS.with(|p| p.borrow().get(&vendor_id))
}

#[query]
pub fn get_vendors() -> Vec<Vendor> {
    let mut vendors: Vec<Vendor> = Vec::new();

    VENDORS.with(|p| {
        for (k, v) in p.borrow().iter() {
            let vendr = Vendor {
                name: v.name,
                logo: v.logo,
                tags: v.tags,
                loyalty: v.loyalty,
                coupon: v.coupon,
                ticket: v.ticket,
                vendor_id: k,
            };

            vendors.push(vendr);
        }
    }); 

    vendors
}

#[update]
pub fn remove_vendor(vendor_id: String) {
    VENDORS.with(|p| {p.borrow_mut().remove(&vendor_id)});
}

#[update]
pub fn create_deal(new_deal: Deal) {
    let res = DEALS.with(|p| p.borrow_mut().insert(new_deal.deal_id.to_string(), new_deal));
}

#[query]
pub fn get_deal(deal_id: String) -> Option<Deal> {
    DEALS.with(|p| p.borrow().get(&deal_id))
}

#[query]
pub fn get_deals() -> Vec<Deal> {
    let mut deals: Vec<Deal> = Vec::new();

    DEALS.with(|p| {
        for (k, v) in p.borrow().iter() {
            let deal = Deal {          
                deal_type: v.deal_type,
                headline: v.headline,
                description: v.description,
                threshold: v.threshold,
                discount: v.discount,
                discount_type: v.discount_type,
                image: v.image,
                vendor_metadata: v.vendor_metadata,
                deal_id: k,
            };

            deals.push(deal);
        }
    }); 

    deals
}

#[update]
pub fn remove_deal(deal_id: String) {
    DEALS.with(|p| {p.borrow_mut().remove(&deal_id)});
}

#[update]
pub fn create_user(new_user: User) {
    let res = USERS.with(|p| p.borrow_mut().insert(new_user.user_id.to_string(), new_user));
}

#[query]
pub fn get_user(user_id: String) -> Option<User> {
    USERS.with(|p| p.borrow().get(&user_id))
}

#[query]
pub fn get_users() -> Vec<User> {
    let mut users: Vec<User> = Vec::new();

    USERS.with(|p| {
        for (k, v) in p.borrow().iter() {
            let user = User {          
                deals: v.deals,
                user_id: k,
            };

            users.push(user);
        }
    }); 

    users
}

#[update]
pub fn remove_user(user_id: String) {
    USERS.with(|p| {p.borrow_mut().remove(&user_id)});
}

#[query]
pub fn get_relevant_deals(tag_id: String, user_id: String) -> Vec<Deal> {
    let order = ORDERS.with(|p| p.borrow().get(&tag_id)).unwrap();
    let user = USERS.with(|p| p.borrow().get(&user_id)).unwrap();    //.unwrap();
    
    let mut deals: Vec<Deal> = Vec::new();
    for v in user.deals.iter() {
        let deal = DEALS.with(|p| p.borrow().get(&v.to_string())).unwrap(); 

        if order.order_amount >= deal.threshold && 
            order.vendor_id == deal.vendor_metadata.vendor_id && 
            order.order_type == deal.deal_type {

            deals.push(deal);            
        }
    }

    deals
}
