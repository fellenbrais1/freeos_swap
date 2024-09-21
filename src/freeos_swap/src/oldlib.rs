// lib.rs
// Code to run and manage processes handled by the freeos_swap canister working with the icrc1_ledger canister

// TODO
// Add the freeos_swap actor ID and mint function call to the freeos_manager canister so mint can be called from the manager using the minter
// Also add the timer function calls etc. as necessary so freeos_swap has all it needs to function
// Take out functions from freeos_swap that are only going to be called using freeos_maanger
// Test the auto-burning and auto-minting functions again once the above are done

// NOTES
// At the moment the mint function can only be called from the freeos_swap canister
// The burn function can only be called from freeos_manager by specifying the Principal of the account to burn from

// CODE START

// import Principal "mo:base/Principal";
// import Blob "mo:base/Blob";
// import Int "mo:base/Int";
// import Nat "mo:base/Nat";
// import Nat64 "mo:base/Nat64";
// import Text "mo:base/Text";
// import Error "mo:base/Error";
// import Time "mo:base/Time";
// import Timer "mo:base/Timer";
// import Debug "mo:base/Debug";
// import Result "mo:base/Result";

// import {JSON; Candid; CBOR;} "mo:serde"; 
// import UrlEncoded "mo:serde";

//IMPORTS ************************************************************

use candid::{CandidType, Nat, Principal};
use serde::Deserialize;
use blob::Blob;
use std::cell::RefCell;
use ic_cdk::*;
use ic_cdk::api::call::call_raw;
use std::convert::TryInto;
use ic_cdk::storage;
// use std::sync::LazyLock;

// TYPES *************************************************************

type Subaccount = Blob;
type Tokens = u64;
type Timestamp = u64;
type BlockIndex = u64;

static mut TRANSFER_AMOUNT: Option<Tokens> = None;
static mut TRANSFER_FEE: Option<Tokens> = None;

// Custom type for the generation of Timestamp values
type Time = u64;

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Account {
    owner: Principal,
    subaccount: Option<[u8; 32]>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum Result<Ok, Err> {
    Ok(Ok),
    Err(Err),
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

#[derive(Clone, Debug, CandidType, Deserialize)]
enum TransferResult {
    Ok(Nat),
    Err(TransferError),
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

// Holds the value of the ledger_client
thread_local! {
    static LEDGER_CLIENT: RefCell<Option<LedgerClient>> = RefCell::new(None);
}

#[derive(CandidType, Deserialize)]
struct GreetingParams {
    to_principal: Principal,
}

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
// FUCK OFF
// impl LedgerClient {
//     fn new(canister_id: Principal) -> Self {
//         Self { canister_id }
//     }

//     #[ic_cdk::query]
//     pub async fn balance_of(Self::canister_id: Principal, account: Account) -> Result<Nat, String> {
//         LedgerClient::balance_of();
//         LEDGER_CLIENT.with(|c| {
//             c.set(c.get() + account.value);
//         });
//     }

    // pub async fn balance_of(&self, account: Account) -> Result<Nat, String> {
    //     let call_result: CallResult<(BalanceResult,)> = call::call(self.canister_id, "icrc1_balance_of", (account,)).await;
    //     match call_result {
    //         Ok((balance_result,)) => {
    //             match balance_result.balance {
    //                 Ok(balance) => Ok(balance),
    //                 Err(e) => Err(format!("Error fetching balance: {:?}", e)),
    //             }
    //             let balance = call_result;
    //             return Ok(balance,);
    //         }S
    //         Err(e) => {
    //             return Err(format!("Error calling icrc1_balance_of: {:?}", e));
    //         }
    //     }
    // }
// }


// FUNCTIONS *************************************************************

// Changes the amount that is transferred in minting/ burning etc.
// Can be called by the user
// #[ic_cdk::update]
// pub fn set_transfer_amount(amount: u64) -> u64 {
//     let transfer_amount = amount;
//     return transfer_amount;
// }

// #[ic_cdk::query]
// pub fn show_transfer_amount() -> u64 {
//     return transfer_amount;
// }

// // Changes the fee exacted on a transaction (default is 0).
// // Can be called by the user\
// #[ic_cdk::update]
// pub fn set_fee(amount: u64) -> u64 {
//     let transfer_fee = amount;
//     return transfer_fee;
// }

// #[ic_cdk::query]
// pub fn show_fee() -> u64 {
//     return transfer_fee;
// }


// Prints the Principal of the caller, this can be mothballed now 
// Can be called by the user
// #[ic_cdk::query]
// pub fn who_am_i() -> String {
//     let username = String::from(username());
//     print!("You are '{}'", &username);
//     return username;
// }

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
    
    // TLC
//     #[ic_cdk::init]
//     fn init() {
//         let ledger_canister_id = Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai")
//         .expect("Invalid ledger canister ID");
//     let client = LedgerClient::new(ledger_canister_id);
//     LEDGER_CLIENT.with(|rc| *rc.borrow_mut() = Some(client));
// }
// let actor = LiftActor { principal: Principal.from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap() };
// let greeting = actor.greet("Alice").await;
// println!("{}", greeting);

//     icrc1_transfer : (TransferArg) -> async TransferResult;
//     icrc1_balance_of : (Account) -> async Nat;
//   };

// async fn interact_with_ledger() -> Result<(), String> {
    //     let ledger_canister_id = Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai")
    //     .map_err(|e|| format!("Invalid Principal: {}", e))?;
    // }

    // VARIABLES *************************************************************
    
    // Defines the to_principal value here to make it easier to update and be used
    // This could be done using '{ caller } / msg.caller' in the future but I haven't been able to get it to work yet
    // Identity: Jesper
    let hard_coded_to_principal = Principal::from_text("tog4r-6yoqs-piw5o-askmx-dwu6g-vncjf-y7gml-qnkb2-yhuao-2cq3c-2ae").expect("Oh dear");
    // Identity: testytester
    // let hardCodedPrincipal = Principal.fromText("stp67-22vw7-sgmm7-aqsla-64hid-auh7e-qjsxr-tr3q2-47jtb-qubd7-6qe");
    
    // Defines the default value of to_principal to the value of hard_coded_to_principal
    let to_principal : Principal = hard_coded_to_principal;
    
    // Variables needed for the auto-minting process
    static mut MINT_TIMER: u64 = 0;
    static mut  IS_MINTING: bool = false;
    static mut MINT_STOP: bool = true;
    
    // Variables needed for the auto-burning process
    static mut BURN_TIMER: u64 = 0;
    static mut IS_BURNING: bool = false;
    static mut BURN_STOP: bool = true;
    
    // Variables to be used to set the contents of the transferArgs for mint and burn functions etc.
    // let mut transfer_amount: Tokens = 50000;
    // let mut transfer_fee: Tokens = 0;
    
    // This doesn't fucking work either
    // Get the Principal of this canister for use in burning
    // const MINTER_PRINCIPAL_STRING: &str = "aaaaa-aa"; // Principal of the current canister
    // static mut MINTER_PRINCIPAL: Principal = Principal::from_text(MINTER_PRINCIPAL_STRING).expect("Something went wrong");

    // Of course this doesn't work because it fucks up all of the other types
    // const MINTER_PRINCIPAL_STRING: &str = "aaaaa-aa"; // Principal of the current canister
    // static MINTER_PRINCIPAL: LazyLock<Principal> = LazyLock::new(|| {
    //     Principal::from_text(MINTER_PRINCIPAL_STRING).expect("Something went wrong")
    // });
    
    // Creating the account type variable to use in the burn() function
    // static mut MINTER_ACCOUNT: Account = Account { 
    // owner: MINTER_PRINCIPAL,
    // subaccount: None,
    // };
    
    // JSON - This approach does not work, you cannot assign the contents of a file directly to a variable like this
    // let testData = "./data.json";
    
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
    
    #[ic_cdk::update]
    pub fn set_transfer_amount(amount: u64) -> u64 {
        unsafe {
            TRANSFER_AMOUNT = Some(amount);
            return TRANSFER_AMOUNT.unwrap();
        }
    }

    #[ic_cdk::query]
    pub fn show_transfer_amount() -> u64 {
        unsafe {
            return TRANSFER_AMOUNT.unwrap();
        }
    }

    // Changes the fee exacted on a transaction (default is 0).
    // Can be called by the user\
    #[ic_cdk::update]
    pub fn set_fee(amount: u64) -> u64 {
        unsafe {
            TRANSFER_FEE = Some(amount);
            return TRANSFER_FEE.unwrap();
        }
    }

    #[ic_cdk::query]
    pub fn show_transfer_fee() -> u64 {
        unsafe {
            return TRANSFER_FEE.unwrap();
        }
    }

}

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
