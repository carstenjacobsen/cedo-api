mod bitcoin_api;
mod bitcoin_wallet;
mod ecdsa_api;
mod types;
mod vendors;

use ic_stable_structures::{
    DefaultMemoryImpl, 
    StableBTreeMap,
    memory_manager::MemoryId,
    memory_manager::MemoryManager,
    memory_manager::VirtualMemory,
};

use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use ic_cdk::println;
use pluto::{
    http::{HttpRequest, RawHttpRequest, RawHttpResponse, HttpResponse, HttpServe},
    http_serve,
    router::Router,
};

use serde::{Serialize, Deserialize};
use serde_json::json;

use candid::Principal;
use ic_cdk::api::call::call_with_payment;
use ic_cdk::api::management_canister::bitcoin::{
    BitcoinNetwork, GetBalanceRequest, GetCurrentFeePercentilesRequest, GetUtxosRequest,
    GetUtxosResponse, MillisatoshiPerByte, Satoshi, SendTransactionRequest,
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static NETWORK: Cell<BitcoinNetwork> = Cell::new(BitcoinNetwork::Testnet);  
    static DERIVATION_PATH: Vec<Vec<u8>> = vec![];

    static KEY_NAME: RefCell<String> = RefCell::new(String::from(""));

    static ROUTER: RefCell<Router>  = RefCell::new(setup());

    static MEM_MAN_BITCOIN: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static BITCOIN: RefCell<StableBTreeMap<String, String, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEM_MAN_BITCOIN.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

#[init]
pub async fn init(network: BitcoinNetwork) {
    NETWORK.with(|n| n.set(network));

    KEY_NAME.with(|key_name| {
        key_name.replace(String::from(match network {
            BitcoinNetwork::Regtest => "dfx_test_key",
            BitcoinNetwork::Mainnet | BitcoinNetwork::Testnet => "test_key_1",
        }))
    });

    let network = NETWORK.with(|n| n.get());
    let balance = "sdfsdfsdf".to_string();
}

#[update]
pub async fn get_balance(address: String) -> u64 {
    let network = NETWORK.with(|n| n.get());
    let balance = bitcoin_api::get_balance(network, address).await;
    BITCOIN.with(|p| p.borrow_mut().insert("bitcoin".to_string(), balance.to_string()));

    balance
}

#[update]
pub async fn get_utxos(address: String) -> GetUtxosResponse {
    let network = NETWORK.with(|n| n.get());
    bitcoin_api::get_utxos(network, address).await
}

#[update]
pub async fn get_current_fee_percentiles() -> Vec<MillisatoshiPerByte> {
    let network = NETWORK.with(|n| n.get());
    bitcoin_api::get_current_fee_percentiles(network).await
}

#[update]
pub async fn get_p2pkh_address() -> String {
    let derivation_path = DERIVATION_PATH.with(|d| d.clone());
    let key_name = KEY_NAME.with(|kn| kn.borrow().to_string());
    let network = NETWORK.with(|n| n.get());

    bitcoin_wallet::get_p2pkh_address(network, key_name, derivation_path).await
}

#[update]
pub async fn send(request: types::SendRequest) -> String {
    let derivation_path = DERIVATION_PATH.with(|d| d.clone());
    let network = NETWORK.with(|n| n.get());
    let key_name = KEY_NAME.with(|kn| kn.borrow().to_string());
    let tx_id = bitcoin_wallet::send(
        network,
        derivation_path,
        key_name,
        request.destination_address,
        request.amount_in_satoshi,
    )
    .await;

    tx_id.to_string()
}

#[query]
async fn http_request(req: RawHttpRequest) -> RawHttpResponse {
    bootstrap(http_serve!(), req).await
}

#[update]
async fn http_request_update(req: RawHttpRequest) -> RawHttpResponse {
    bootstrap(http_serve!(), req).await
}

async fn bootstrap(mut app: HttpServe, req: RawHttpRequest) -> RawHttpResponse {
    let router = ROUTER.with(|r| r.borrow().clone());
    app.set_router(router);
    app.serve(req).await
}

pub(crate) fn setup() -> Router {
    let mut router = Router::new();

    router.get("/deals/:tag_id/:user_id", false, |_req: HttpRequest| async move {
        let tag_id: String = _req.params.get("tag_id").unwrap().to_string();
        let user_id: String = _req.params.get("user_id").unwrap().to_string();
        let deals = vendors::get_relevant_deals(tag_id, user_id);
        let json = serde_json::to_string(&deals).expect("Serialization failed");

        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 200,
                "message": format!("{json}"),
            })
            .into(),
        })
    });

    router.get("/order/:tag_id", false, |_req: HttpRequest| async move {
        let tag_id: String = _req.params.get("tag_id").unwrap().to_string();
        let order = vendors::get_order(tag_id);
        let json = serde_json::to_string(&order).expect("Serialization failed");

        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 200,
                "message": format!("{json}"),
            })
            .into(),
        })
    });

    router.get("/bitcoin/balance/:address", false, |_req: HttpRequest| async move {
        let address: String = _req.params.get("address").unwrap().to_string();
        let balance = BITCOIN.with(|p| p.borrow().get(&"bitcoin".to_string())).unwrap().to_string();

        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: json!({
                "statusCode": 200,
                "message": balance,  
            })
            .into(),
        })
    });

    router
}

#[pre_upgrade]
fn pre_upgrade() {
    let network = NETWORK.with(|n| n.get());
    ic_cdk::storage::stable_save((network,)).expect("Saving network to stable store must succeed.");
}

#[post_upgrade]
fn post_upgrade() {
    let network = ic_cdk::storage::stable_restore::<(BitcoinNetwork,)>()
        .expect("Failed to read network from stable memory.")
        .0;

    init(network);
}
