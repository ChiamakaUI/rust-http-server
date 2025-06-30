use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Keypair,
};
use std::str::FromStr;

use crate::schema::{AccountMetaResponse, InstructionResponse};

// Parse a base58 string into a Pubkey
pub fn parse_pubkey(s: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(s).map_err(|e| format!("Invalid public key: {}", e))
}

// Parse a base58 secret key into a Keypair
pub fn parse_keypair(secret: &str) -> Result<Keypair, String> {
    bs58::decode(secret)
        .into_vec()
        .map_err(|e| format!("Invalid base58 encoding: {}", e))
        .and_then(|bytes| {
            if bytes.len() != 64 {
                return Err("Secret key must be 64 bytes".to_string());
            }
            let mut secret_bytes = [0u8; 64];
            secret_bytes.copy_from_slice(&bytes);
            Ok(Keypair::from_bytes(&secret_bytes)
                .map_err(|e| format!("Invalid keypair: {}", e))?)
        })
}

pub fn instruction_to_response(instruction: Instruction) -> InstructionResponse {
    InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .into_iter()
            .map(|meta| AccountMetaResponse {
                pubkey: meta.pubkey.to_string(),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        instruction_data: BASE64.encode(&instruction.data),
    }
}