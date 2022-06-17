use rocket::serde::json::Json;
use tonic::Request;
use zcash_primitives::transaction::components::amount::DEFAULT_FEE;
use crate::{db, Error, get_appstore, scan_blocks, wallet};
use crate::lw_rpc::compact_tx_streamer_client::CompactTxStreamerClient;
use crate::lw_rpc::Empty;
use crate::rpc::data::*;

pub mod data;

#[post("/create_account", data = "<request>")]
pub async fn create_account(
    request: Json<CreateAccountRequest>
) -> Result<Json<CreateAccountResponse>, Error> {
    let request = request.into_inner();
    let response = wallet::create_account(request.label).await?;
    Ok(Json(response))
}

#[post("/create_address", data = "<request>")]
pub async fn create_address(
    request: Json<CreateAddressRequest>,
) -> Result<Json<CreateAddressResponse>, Error> {
    let request = request.into_inner();
    let response = wallet::create_address(request.label, request.account_index).await?;
    Ok(Json(response))
}

#[post("/get_accounts", data = "<_request>")]
pub async fn get_accounts(
    _request: Json<GetAccountsRequest>,
) -> Result<Json<GetAccountsResponse>, Error> {
    let app = get_appstore();
    let height = crate::chain::get_height().await?;
    let confirmations = app.config.confirmations;
    let accounts = db::Db::get_accounts(get_appstore().store.clone(), height, confirmations)?;
    let total_balance: u64 = accounts.iter().map(|sa| sa.balance).sum();
    let total_unlocked_balance: u64 = accounts.iter().map(|sa| sa.unlocked_balance).sum();
    let rep = GetAccountsResponse {
        subaddress_accounts: accounts,
        total_balance,
        total_unlocked_balance,
    };
    Ok(Json(rep))
}

#[post("/get_transfer_by_txid", data = "<request>")]
pub async fn get_transaction(
    request: Json<GetTransactionByIdRequest>,
) -> Result<Json<GetTransactionByIdResponse>, Error> {
    let request = request.into_inner();
    let mut txid = hex::decode(&request.txid)?;
    txid.reverse();
    let app = get_appstore();
    let height = crate::chain::get_height().await?;
    let confirmations = app.config.confirmations;
    let tx = db::Db::get_transaction(get_appstore().store.clone(), request.account_index, &txid, height, confirmations)?;
    Ok(Json(tx))
}

#[post("/get_transfers", data = "<request>")]
pub async fn get_transfers(
    request: Json<GetTransfersRequest>,
) -> Result<Json<GetTransfersResponse>, Error> {
    let request = request.into_inner();
    let app = get_appstore();
    let height = crate::chain::get_height().await?;
    let confirmations = app.config.confirmations;
    let transfers = db::Db::get_transfers(get_appstore().store.clone(), height, request.account_index, &request.subaddr_indices, confirmations)?;
    let rep = GetTransfersResponse {
        r#in: transfers,
    };
    Ok(Json(rep))
}

#[post("/get_fee_estimate", data = "<_request>")]
pub fn get_fee_estimate(_request: Json<GetFeeEstimateRequest>) -> Result<Json<GetFeeEstimateResponse>, Error> {
    let rep = GetFeeEstimateResponse {
        fee: DEFAULT_FEE.into()
    };
    Ok(Json(rep))
}

#[post("/get_height", data = "<_request>")]
pub async fn get_height(_request: Json<GetHeightRequest>) -> Result<Json<GetHeightResponse>, Error> {
    let height = crate::chain::get_height().await?;
    let rep = GetHeightResponse {
        height,
    };
    Ok(Json(rep))
}

#[post("/sync_info", data = "<_request>")]
pub async fn sync_info(_request: Json<SyncInfoRequest>) -> Result<Json<SyncInfoResponse>, Error> {
    let height = crate::chain::get_height().await?;
    let rep = SyncInfoResponse {
        target_height: height,
        height, // Pirate lightwalletd does not return estimate height
    };
    Ok(Json(rep))
}

#[post("/request_scan")]
pub async fn request_scan() -> Result<(), Error> {
    scan_blocks().await?;
    Ok(())
}

