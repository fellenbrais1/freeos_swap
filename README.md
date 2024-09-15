# `freeos_swap`

Welcome to your new `freeos_swap` project and to the Internet Computer development community. By default, creating a new project adds this README and some template files to your project directory. You can edit these template files to customize your project and to include your own code to speed up the development cycle.

To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

To learn more before you start working with `freeos_swap`, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/current/developer-docs/setup/deploy-locally)
- [SDK Developer Tools](https://internetcomputer.org/docs/current/developer-docs/setup/install)
- [Rust Canister Development Guide](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/current/developer-docs/backend/candid/)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd freeos_swap/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.

### Note on frontend environment variables

If you are hosting frontend code somewhere without using DFX, you may need to make one of the following adjustments to ensure your project does not fetch the root key in production:

- set`DFX_NETWORK` to `ic` if you are using Webpack
- use your own preferred method to replace `process.env.DFX_NETWORK` in the autogenerated declarations
  - Setting `canisters -> {asset_canister_id} -> declarations -> env_override to a string` in `dfx.json` will replace `process.env.DFX_NETWORK` with the string in the autogenerated declarations
- Write your own `createActor` constructor



## SETUP INSTRUCTIONS

Using icrc1_ledger and freeos_swap canisters
This set of canisters creates LIFT (Lift Cash) token. Then it demonstrates how to mint by calling the freeos_swap canister's mint function. There is also an auto-minting capability that be turned off or on by calling 'startMinting()' and 'stopMinting()' on the 'freeos_swap' canister.

N.B. At the moment you will have to create your own ids, e.g. for the recipient user specified in freeos_swap main.mo Later this will hopefully be automated, at least to some extent.



## Step 1: Download the latest icrc1_ledger wasm and did file

(! This step hasn't been needed yet, but could be needed to keep a proper deployment up to date)

Run the (Linux/Mac) command: `source ./download_latest_icrc1_ledger.sh`

The files ('icrc1_ledger.did' and 'icrc1_ledger.wasm.gz') should be placed in the 'src/lift' directory.



## Step 2: Build all of the canisters

(! This step hasn't been needed yet, but could be dependent on the OS and other factors, for me it is unnecessary)

Run the command: `dfx build`



## Step 3: Deploy the freeos_swap canister

Run the command: `dfx deploy freeos_swap`

Take note of the canister id generated for 'freeos_swap'. This is the 'minter principal' required by the 'icrc1_ledger' canister. The 'freeos_swap' canister will become the only entity capable of minting tokens on the 'icrc1_ledger' canister.



## Step 4: Set up the environment variables used in step 5:

Edit the 'set_env.sh' file to set 'MINTER' equal to the 'freeos_swap' canister id.

The line we need to change should look like this: `export MINTER=bkyz2-fmaaa-aaaaa-qaaaq-cai` (With the Principal id of your instance of the 'freeos_swap' canister)

We also need to the change the line that defines the hardCodedToPrincipal to default the toPrincipal address to the Principal you will be using to access it (to make things easier make this the same one you deploy canisters as) `let hardCodedToPrincipal = Principal.fromText("tog4r-6yoqs-piw5o-askmx-dwu6g-vncjf-y7gml-qnkb2-yhuao-2cq3c-2ae");` (With the Principal id of the account that will be holding the balance)

Then run this shell file using this (Linux/Mac) command: `source ./set_env.sh`

This will set up the variables needed for the next step, and will also deploy the freeos_manager canister.



## Step 5: Command to deploy the icrc1_ledger canister:

Make sure you are using the same identity you deployed the 'freeos_swap' canister from or you will encounter an error here.

Run this shell file using this (Linux/Mac) command: `source ./deploy_icrc1.sh`

Alternatively, we can run the same command as in the shell file in the CLI (not including the opening and closing triple backticks):

```
dfx deploy icrc1_ledger --specified-id mxzaz-hqaaa-aaaar-qaada-cai --argument "(variant {Init =
record {
token_symbol = \"${TOKEN_SYMBOL}\";
     token_name = \"${TOKEN_NAME}\";
minting_account = record { owner = principal \"${MINTER}\" };
     transfer_fee = ${TRANSFER_FEE};
     metadata = vec {};
     feature_flags = opt record{icrc2 = ${FEATURE_FLAGS}};
     initial_balances = vec { record { record { owner = principal \"${DEFAULT}\"; }; ${PRE_MINTED_TOKENS}; }; };
     archive_options = record {
         num_blocks_to_archive = ${NUM_OF_BLOCK_TO_ARCHIVE};
         trigger_threshold = ${TRIGGER_THRESHOLD};
         controller_id = principal \"${ARCHIVE_CONTROLLER}\";
cycles_for_archive_creation = opt ${CYCLE_FOR_ARCHIVE_CREATION};
};
}
})"
```

This will deploy the 'icrc1_ledger' canister with all of the arguments needed for proper connectedness and operation.



## Step 6: Call freeos_swap mint function to transfer 50,000 tokens from the minter account to user blwz3-4wsku-3otjv-yriaj-2hhdr-3gh3e-x4z7v-psn6e-ent7z-eytoo-mqe

Run the following canister call in the CLI: `dfx canister call freeos_swap mint '()'`

We should get the response if it is working as intended: `(variant { Ok = 1 : nat })`

We can call any of the public functions on the 'freeos_swap' and 'icrc1_ledger' canisters, but to test more easily, use the Candid UI links generated.