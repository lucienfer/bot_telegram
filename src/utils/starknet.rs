/*
    Module for all fonction who read the Starknet blockchain
*/
pub mod read {
    use starknet::{
        core::types::{BlockId, BlockTag, Felt, FunctionCall},
        macros::selector,
        providers::{
            jsonrpc::{HttpTransport, JsonRpcClient},
            Provider, Url,
        },
    };
    use serde_json::Value;

/* 
    Get balance of the token given in param, you can specified if you want the balance value in $ or just the balance of the specified token

    Todo: - Error handling
*/
    pub async fn get_balance_erc20(token_addr: String, account_addr: String, usd: bool) -> i64 {
        let provider = JsonRpcClient::new(HttpTransport::new(
            Url::parse(&std::env::var("RPC_API_URL").expect("RPC_API_URL must be set")).unwrap(),
        ));
        let client = reqwest::Client::new();
    
        let token_addr_hex = Felt::from_hex(&token_addr).unwrap();
        let account_addr_hex = Felt::from_hex(&account_addr).unwrap();
    
        let call_result =
            provider
                .call(
                    FunctionCall {
                        contract_address: token_addr_hex,
                        entry_point_selector: selector!("balanceOf"),
                        calldata: vec![
                            account_addr_hex
                        ],
                    },
                    BlockId::Tag(BlockTag::Latest),
                )
                .await;
        
        match call_result {
            Ok(value) => {
                let divisor = Felt::from_dec_str("1000000000000000000").unwrap(); // TODO: make fonction get decimal depend on the token
                let result = value[0].floor_div(&divisor.try_into().unwrap());
                let tmp: String = result.to_string();
                let balance: i64 = tmp.parse::<i64>().expect("REASON");
                if !usd {
                    return balance;
                }

                let response = client
                    .get("https://api.dexscreener.com/latest/dex/pairs/starknet/0x03b405a98c9e795d427fe82cdeeeed803f221b52471e3a757574a2b4180793ee-0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d-1020847100762815390390123822295304634-5982-0x0")
                    .send()
                    .await.unwrap();
                if response.status().is_success() {
                    let json: Value = response.json().await.unwrap();
                    let price = json["pairs"][0]["priceUsd"].as_str().unwrap();
                    match price.parse::<f64>() {
                        Ok(float_value) => {
                            let balance_usd = balance as f64 * float_value;
                            return balance_usd as i64;
                        },
                        Err(_) => 0,
                    };
            }
            },
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
        return 0;
    }

/*
    Verify if the address given in param is an starknet valid address.
*/
    pub async fn verify_addr(account_addr: String) -> bool {
        let provider = JsonRpcClient::new(HttpTransport::new(
            Url::parse(&std::env::var("RPC_API_URL").expect("RPC_API_URL must be set")).unwrap(),
        ));
        let hex_addr = Felt::from_hex(&account_addr);

        match hex_addr {
            Ok(value) => {
            let call_result =
                provider
                    .call(
                        FunctionCall {
                            contract_address: value,
                            entry_point_selector: selector!("get_public_key"),
                            calldata: vec![],
                        },
                        BlockId::Tag(BlockTag::Latest),
                    )
                    .await;
                match call_result {
                    Ok(_value) => {
                        return true;
                    }
                    Err(_) => {
                        return false;
                    }
                }
            }
            Err(_) => {
                return false;
            }
        }
    }
}