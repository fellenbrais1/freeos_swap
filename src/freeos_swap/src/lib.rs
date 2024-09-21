// lib.rs
// Clean version of work in progresss

// CODE START

//IMPORTS ************************************************************

use api::management_canister::bitcoin::GetCurrentFeePercentilesRequest;
use candid::{CandidType, Nat, Principal};
use serde::Deserialize;
use blob::Blob;
use std::cell::RefCell;
use std::f32::MIN;
use std::mem::transmute;
use std::str::FromStr;
use ic_cdk::*;
use ic_cdk::api::call::call_raw;
use std::convert::TryInto;
use ic_cdk::storage;

// TYPES *************************************************************

type Subaccount = Blob;
type Tokens = u64;
type Timestamp = u64;
type BlockIndex = u64;

// Custom type for the generation of Timestamp values
type Time = u64;

const INITIAL_TRANSFER_AMOUNT: Tokens = 50000;
const INITIAL_TRANSFER_FEE: Tokens = 0;

// Defines the to_principal value here to make it easier to update and be used
// Identity: Jesper
const INITIAL_TO_PRINCIPAL_STRING: &str = "tog4r-6yoqs-piw5o-askmx-dwu6g-vncjf-y7gml-qnkb2-yhuao-2cq3c-2ae.";
// Identity: testytester
// const INITIAL_TO_PRINCIPAL_STRING: &str = "stp67-22vw7-sgmm7-aqsla-64hid-auh7e-qjsxr-tr3q2-47jtb-qubd7-6qe";

#[derive(Clone, Debug, CandidType, serde::Deserialize)]
pub struct SendInfo {
    proton_account: String,
    ic_principal: Principal,
    amount: u64,
    utc_time: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct TimerObject {
    mint_timer: u64,
    is_minting: bool,
    mint_stop: bool,  
    burn_timer: u64,
    id_burning: bool,
    burn_stop: bool,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct TransferArg {
    from_subaccount: Option<[u8; 32]>,
    to: Account,
    amount: Nat,
    fee: Option<Nat>,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
}
#[derive(Clone, Debug, CandidType, Deserialize)]
struct Account {
    owner: Principal,
    subaccount: Option<[u8; 32]>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct GreetingParams {
    to_principal: Principal,
}

// JSON - New custom type to create JSON records
#[derive(Clone, Debug, CandidType, Deserialize)]
struct JsonRecord {
    proton_account: String,
    ic_principal_id: Principal,
    amount: u64,
    date_time: u64
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct LedgerClient {
    canister_id: Principal,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum TransferResult {
    Ok(Nat),
    Err(TransferError),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum Result<Ok, Err> {
    Ok(Ok),
    Err(Err),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum CustomResult {
    Ok (u64),
    Err(String),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

// Holds the value of the ledger_client
thread_local! {
    static LEDGER_CLIENT: RefCell<Option<LedgerClient>> = RefCell::new(None);
}

// JESPER - Not sure if needed?
#[update]
async fn greet_other_canister(ledger_canister_id: Principal, to_principal: Principal) -> String {
    let args = GreetingParams { to_principal };
    
    match call_raw(
        ledger_canister_id,
        "icrc1_balance_of",
        &candid::encode_one(args).unwrap(),
        0
    ).await {
        Ok(response) => {
            let balance: String = candid::decode_one(&response).unwrap();
            return balance
        },
        Err((code, msg)) => {
            format!("Error: {:?} - {}", code, msg)
        }
    }
}

// FUNCTIONS *************************************************************

// Changes the toPrincipal to mint to /burn from as needed
// Later we could potentially use this to iterate over a range of Principals and change the to address each time
// Can be called by the user
#[ic_cdk::update]
pub fn set_to_principal(set_principal : Principal) -> Principal {
    let to_principal = set_principal;
    print!("To Principal set to {}", Principal::to_text(&to_principal));
    return to_principal;
}

fn main() {
    
    // VARIABLES *************************************************************
    
    use ic_cdk::storage;
    
    // Defines the default value of to_principal to the value of hard_coded_to_principal
    let to_principal : Principal = Principal::from_text(INITIAL_TO_PRINCIPAL_STRING).unwrap();
    
    let timer_object: TimerObject = TimerObject {
        mint_timer: 0,
        is_minting: false,
        mint_stop: true,  
        burn_timer: 0,
        id_burning: false,
        burn_stop: true,   
    };
    
    // Variables to be used to set the contents of the transferArgs for mint and burn functions etc.
    // #[ic_cdk::storage]
    let mut transfer_amount: Tokens = INITIAL_TRANSFER_AMOUNT;
    let mut transfer_fee: Tokens = INITIAL_TRANSFER_FEE;
    
    // Get the Principal of this canister for use in burning
    // static mut MINTER_PRINCIPAL: Principal = "";
    let minter_principal: Principal = Principal::from_text("aaaaa-aa").expect("Something went wrong");
    
    // Creating the account type variable to use in the burn() function
    let minter_account: Account = Account { 
    owner: minter_principal,
    subaccount: None,
    };
    
    let new_record: JsonRecord = JsonRecord {
        proton_account: String::from("tommccann"),
        ic_principal_id: Principal::from_text("gpurw-f4h72-qwdnm-vmexj-xnhww-us2kt-kbiua-o3y4u-bzduw-qhb7a-jqe").expect("Oh dear"),
        amount: 100,
        date_time: 1725805695,
    };
    
    let new_record_2: JsonRecord = JsonRecord {
        proton_account: String::from("judetan"),
        ic_principal_id: Principal::from_text("22gak-zasla-2cj5r-ix2ds-4kaxw-lrgtq-4zjul-mblvf-gkhsi-fzu3j-cae").expect("Oh dear"),
        amount: 40,
        date_time: 1725805791,
    };
    
    // JSON - Tried to just make an array of jsonRecord values to mimic a JSON response, looks promising if json values can be extracted into this format
    let json_array: [JsonRecord; 2] = [
        new_record,
        new_record_2,
    ];
    
    let json_record_keys = ["proton_account", "ic_principal", "amount", "date_time"];
    
    let mut current_send_object: SendInfo = SendInfo {
        proton_account: String::from(""),
        ic_principal: to_principal,
        amount: 0,
        utc_time: 0,
    };

    #[ic_cdk::query]
    fn get_my_struct(foo: SendInfo) -> SendInfo {
        crate::println!("{:#?}", foo);
        foo
    }
    
    #[ic_cdk::query]
    pub fn experimental(c: SendInfo) -> u64 {
        let object_to_print = get_my_struct(&current_send_object);
        object_to_print.amount
    }

    #[ic_cdk::update]
    pub fn set_transfer_amount(amount: u64, current_send_object: SendInfo) -> u64 {
        current_send_object.amount = amount;
        return current_send_object.amount;
    }

    // #[ic_cdk::query]
    // pub fn show_transfer_amount(amount: u64) -> u64 {
    //     let t_amount = transfer_amount;
    //     return t_amount;
    // }

    // // Changes the fee exacted on a transaction (default is 0).
    // // Can be called by the user
    // #[ic_cdk::update]
    // fn set_fee(set_amount: u64) -> u64 {
    //     let transfer_fee: u64 = set_amount;
    //     return transfer_fee;
    // }
            
    // #[ic_cdk::query]
    // pub fn show_transfer_fee(fee: &transfer_fee) -> u64 {
    //     return transfer_fee;
    // }

}

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
