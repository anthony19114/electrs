use bitcoin::blockdata::script::Instruction::PushBytes;
use bitcoin::consensus::encode::serialize;
use bitcoin::Script;
use elements::confidential::Value;
use elements::{AssetIssuance, Proof};

use hex;

use crate::chain::Network;
use crate::util::{get_script_asm, script_to_address};

#[derive(Serialize, Deserialize)]
pub struct BlockProofValue {
    challenge: Script,
    challenge_asm: String,
    solution: Script,
    solution_asm: String,
}

impl From<&Proof> for BlockProofValue {
    fn from(proof: &Proof) -> Self {
        BlockProofValue {
            challenge_asm: get_script_asm(&proof.challenge),
            challenge: proof.challenge.clone(),
            solution_asm: get_script_asm(&proof.solution),
            solution: proof.solution.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IssuanceValue {
    pub is_reissuance: bool,
    pub asset_blinding_nonce: Option<String>,
    pub asset_entropy: Option<String>,
    pub assetamount: Option<u64>,
    pub assetamountcommitment: Option<String>,
    pub tokenamount: Option<u64>,
    pub tokenamountcommitment: Option<String>,
}

impl From<&AssetIssuance> for IssuanceValue {
    fn from(issuance: &AssetIssuance) -> Self {
        let zero = [0u8; 32];
        let is_reissuance = issuance.asset_blinding_nonce != zero;

        IssuanceValue {
            is_reissuance,
            asset_blinding_nonce: if is_reissuance {
                Some(hex::encode(issuance.asset_blinding_nonce))
            } else {
                None
            },
            asset_entropy: if issuance.asset_entropy != zero {
                Some(hex::encode(issuance.asset_entropy))
            } else {
                None
            },
            assetamount: match issuance.amount {
                Value::Explicit(value) => Some(value),
                _ => None,
            },
            assetamountcommitment: match issuance.amount {
                Value::Confidential(..) => Some(hex::encode(serialize(&issuance.amount))),
                _ => None,
            },
            tokenamount: match issuance.inflation_keys {
                Value::Explicit(value) => Some(value / 100000000), // https://github.com/ElementsProject/rust-elements/issues/7
                _ => None,
            },
            tokenamountcommitment: match issuance.inflation_keys {
                Value::Confidential(..) => Some(hex::encode(serialize(&issuance.inflation_keys))),
                _ => None,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PegOutRequest {
    pub genesis_hash: String,
    pub scriptpubkey: Script,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_address: Option<String>,
}

impl PegOutRequest {
    pub fn parse(
        script: &Script,
        parent_network: &Network,
        parent_genesis_hash: &str,
    ) -> Option<PegOutRequest> {
        if !script.is_op_return() {
            return None;
        }

        let nulldata: Vec<_> = script.iter(true).skip(1).collect();
        if nulldata.len() < 2 {
            return None;
        }

        let genesis_hash = if let PushBytes(data) = nulldata[0] {
            let mut data = data.to_vec();
            data.reverse();
            hex::encode(data)
        } else {
            return None;
        };

        let scriptpubkey = if let PushBytes(data) = nulldata[1] {
            Script::from(data.to_vec())
        } else {
            return None;
        };

        if genesis_hash != parent_genesis_hash {
            return None;
        }

        let scriptpubkey_asm = get_script_asm(&scriptpubkey);
        let scriptpubkey_address = script_to_address(&scriptpubkey, parent_network);

        Some(PegOutRequest {
            genesis_hash,
            scriptpubkey,
            scriptpubkey_asm,
            scriptpubkey_address,
        })
    }
}
