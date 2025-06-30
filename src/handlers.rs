
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use solana_sdk::{
    signature::{Keypair, Signature, Signer},
    system_instruction,
};
use spl_token::{
    instruction as token_instruction,
    id as token_program_id,
};

use crate::schema::*;
use crate::utils::*;

// Generate a new keypair
pub async fn generate_keypair() -> impl IntoResponse {
    let keypair = Keypair::new();
    let response = ApiResponse::success(KeypairResponse {
        pubkey: keypair.pubkey().to_string(),
        secret: bs58::encode(keypair.to_bytes()).into_string(),
    });
    (StatusCode::OK, Json(response))
}

// Create token mint instruction
pub async fn create_token(
    Json(payload): Json<CreateTokenRequest>,
) -> impl IntoResponse {
    let mint_authority = match parse_pubkey(&payload.mint_authority) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<InstructionResponse>::error(e))),
    };

    let mint = match parse_pubkey(&payload.mint) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<InstructionResponse>::error(e))),
    };

    let instruction = token_instruction::initialize_mint(
        &token_program_id(),
        &mint,
        &mint_authority,
        None,
        payload.decimals,
    )
    .unwrap();

    let response = ApiResponse::success(instruction_to_response(instruction));
    (StatusCode::OK, Json(response))
}

// Mint tokens
pub async fn mint_token(
    Json(payload): Json<MintTokenRequest>,
) -> impl IntoResponse {
    let mint = match parse_pubkey(&payload.mint) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<InstructionResponse>::error(e))),
    };

    let destination = match parse_pubkey(&payload.destination) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<InstructionResponse>::error(e))),
    };

    let authority = match parse_pubkey(&payload.authority) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<InstructionResponse>::error(e))),
    };

    let instruction = token_instruction::mint_to(
        &token_program_id(),
        &mint,
        &destination,
        &authority,
        &[],
        payload.amount,
    )
    .unwrap();

    let response = ApiResponse::success(instruction_to_response(instruction));
    (StatusCode::OK, Json(response))
}

// Sign a message
pub async fn sign_message(
    Json(payload): Json<SignMessageRequest>,
) -> impl IntoResponse {
    if payload.message.is_empty() || payload.secret.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<SignMessageResponse>::error("Missing required fields".to_string())),
        );
    }

    let keypair = match parse_keypair(&payload.secret) {
        Ok(kp) => kp,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<SignMessageResponse>::error(e))),
    };

    let message_bytes = payload.message.as_bytes();
    let signature = keypair.sign_message(message_bytes);

    let response = ApiResponse::success(SignMessageResponse {
        signature: BASE64.encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: payload.message,
    });

    (StatusCode::OK, Json(response))
}

// Verify a signed message
pub async fn verify_message(
    Json(payload): Json<VerifyMessageRequest>,
) -> impl IntoResponse {
    let pubkey = match parse_pubkey(&payload.pubkey) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<VerifyMessageResponse>::error(e))),
    };

    let signature_bytes = match BASE64.decode(&payload.signature) {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<VerifyMessageResponse>::error(format!("Invalid base64 signature: {}", e))),
            )
        }
    };

    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<VerifyMessageResponse>::error(format!("Invalid signature: {}", e))),
            )
        }
    };

    let message_bytes = payload.message.as_bytes();
    let valid = signature.verify(pubkey.as_ref(), message_bytes);

    let response = ApiResponse::success(VerifyMessageResponse {
        valid,
        message: payload.message,
        pubkey: payload.pubkey,
    });

    (StatusCode::OK, Json(response))
}

// Send SOL
pub async fn send_sol(
    Json(payload): Json<SendSolRequest>,
) -> impl IntoResponse {
    let from = match parse_pubkey(&payload.from) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<InstructionResponse>::error(e))),
    };

    let to = match parse_pubkey(&payload.to) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<InstructionResponse>::error(e))),
    };

    if payload.lamports == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<InstructionResponse>::error("Lamports must be greater than 0".to_string())),
        );
    }

    let instruction = system_instruction::transfer(&from, &to, payload.lamports);
    let response = ApiResponse::success(instruction_to_response(instruction));
    (StatusCode::OK, Json(response))
}

// Send tokens
pub async fn send_token(
    Json(payload): Json<SendTokenRequest>,
) -> impl IntoResponse {
    let destination = match parse_pubkey(&payload.destination) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<InstructionResponse>::error(e))),
    };

    let mint = match parse_pubkey(&payload.mint) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<InstructionResponse>::error(e))),
    };

    let owner = match parse_pubkey(&payload.owner) {
        Ok(pk) => pk,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse::<InstructionResponse>::error(e))),
    };

    let source = owner;
    let destination_token_account = destination;

    let instruction = token_instruction::transfer_checked(
        &token_program_id(),
        &source,
        &mint,
        &destination_token_account,
        &owner,
        &[],
        payload.amount,
        9,
    )
    .unwrap();

    let response = ApiResponse::success(instruction_to_response(instruction));
    (StatusCode::OK, Json(response))
}

// Create all routes
pub fn create_routes() -> Router {
    Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token))
}