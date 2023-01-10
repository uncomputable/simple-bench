use elements::hashes::sha256::Midstate;
use elements::hashes::Hash;
use elements::secp256k1_zkp::Tweak;
use elements::taproot::ControlBlock;
use elements::{
    confidential, AssetId, AssetIssuance, BlockHash, OutPoint, PackedLockTime, Sequence,
    Transaction, TxIn, TxInWitness, TxOut, TxOutWitness,
};
use rand::rngs::ThreadRng;
use rand::Rng;
use simplicity::jet::elements::{ElementsEnv, ElementsUtxo};
use simplicity::jet::{Core, Elements, Jet, JetFailed};
use simplicity::merkle::cmr::Cmr;
use simplicity_sys::c_jets::frame_ffi::c_writeBit;
use simplicity_sys::c_jets::round_u_word;
use simplicity_sys::{CElementsTxEnv, CFrameItem};
use std::sync::Arc;

type JetBuffer = (Vec<usize>, *mut usize, *mut usize, *mut usize, *mut usize);

pub fn init_jet(src_bit_width: usize, tgt_bit_width: usize) -> JetBuffer {
    let a_frame_size = round_u_word(src_bit_width);
    let b_frame_size = round_u_word(tgt_bit_width);

    if a_frame_size == 0 && b_frame_size == 0 {
        panic!("Jet without input nor output");
    }

    let mut src_buf = vec![0usize; a_frame_size + b_frame_size];
    let src_ptr_end = unsafe { src_buf.as_mut_ptr().add(a_frame_size) };
    let src_ptr = src_buf.as_mut_ptr();
    let dst_ptr_begin = unsafe { src_buf.as_mut_ptr().add(a_frame_size) };
    let dst_ptr_end = unsafe { src_buf.as_mut_ptr().add(a_frame_size + b_frame_size) };

    (src_buf, src_ptr_end, src_ptr, dst_ptr_begin, dst_ptr_end)
}

pub fn run_jet<J: Jet>(
    jet: &'static dyn Fn(&mut CFrameItem, CFrameItem, &J::CJetEnvironment) -> bool,
    env: &J::CJetEnvironment,
    src_bit_width: usize,
    tgt_bit_width: usize,
    buffer: &mut JetBuffer,
    rng: &mut ThreadRng,
) -> Result<(), JetFailed> {
    let (ref mut src_buf, src_ptr_end, src_ptr, _, dst_ptr_end) = *buffer;

    let mut a_frame = unsafe { CFrameItem::new_write(src_bit_width, src_ptr_end) };
    for _ in 0..src_bit_width {
        let bit = rng.gen();
        unsafe {
            c_writeBit(&mut a_frame, bit);
        }
    }

    let src_frame = unsafe { CFrameItem::new_read(src_bit_width, src_ptr) };
    let mut dst_frame = unsafe { CFrameItem::new_write(tgt_bit_width, dst_ptr_end) };

    let jet_successful = jet(&mut dst_frame, src_frame, env);
    src_buf.fill(0);

    if jet_successful {
        Ok(())
    } else {
        Err(JetFailed)
    }
}

pub trait DefaultEnv: Jet {
    fn default_env() -> <Self as Jet>::Environment;
}

impl DefaultEnv for Core {
    fn default_env() -> <Self as Jet>::Environment {
        Self::Environment::default()
    }
}

impl DefaultEnv for Elements {
    fn default_env() -> <Self as Jet>::Environment {
        // Copied from rust-simplicity/src/jet/elements/tests.rs

        fn hex_script(s: &str) -> elements::Script {
            let v: Vec<u8> = bitcoin_hashes::hex::FromHex::from_hex(s).unwrap();
            elements::Script::from(v)
        }

        let asset: [u8; 32] = [
            0x23, 0x0f, 0x4f, 0x5d, 0x4b, 0x7c, 0x6f, 0xa8, 0x45, 0x80, 0x6e, 0xe4, 0xf6, 0x77,
            0x13, 0x45, 0x9e, 0x1b, 0x69, 0xe8, 0xe6, 0x0f, 0xce, 0xe2, 0xe4, 0x94, 0x0c, 0x7a,
            0x0d, 0x5d, 0xe1, 0xb2,
        ];
        let tx_id: [u8; 32] = [
            0xeb, 0x04, 0xb6, 0x8e, 0x9a, 0x26, 0xd1, 0x16, 0x04, 0x6c, 0x76, 0xe8, 0xff, 0x47,
            0x33, 0x2f, 0xb7, 0x1d, 0xda, 0x90, 0xff, 0x4b, 0xef, 0x53, 0x70, 0xf2, 0x52, 0x26,
            0xd3, 0xbc, 0x09, 0xfc,
        ];
        let ctrl_blk: [u8; 33] = [
            0xc0, 0xeb, 0x04, 0xb6, 0x8e, 0x9a, 0x26, 0xd1, 0x16, 0x04, 0x6c, 0x76, 0xe8, 0xff,
            0x47, 0x33, 0x2f, 0xb7, 0x1d, 0xda, 0x90, 0xff, 0x4b, 0xef, 0x53, 0x70, 0xf2, 0x52,
            0x26, 0xd3, 0xbc, 0x09, 0xfc,
        ];
        let asset = confidential::Asset::Explicit(AssetId::from_inner(Midstate::from_inner(asset)));
        let tx = Transaction {
            version: 2,
            lock_time: PackedLockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: elements::Txid::from_inner(tx_id),
                    vout: 0,
                },
                sequence: Sequence::ENABLE_LOCKTIME_NO_RBF,
                is_pegin: false,
                // perhaps make this an option in elements upstream?
                asset_issuance: AssetIssuance {
                    asset_blinding_nonce: Tweak::from_inner([0; 32]).expect("tweak from inner"),
                    asset_entropy: [0; 32],
                    amount: confidential::Value::Null,
                    inflation_keys: confidential::Value::Null,
                },
                script_sig: elements::Script::new(),
                witness: TxInWitness {
                    amount_rangeproof: None,
                    inflation_keys_rangeproof: None,
                    script_witness: vec![],
                    pegin_witness: vec![],
                },
            }],
            output: vec![
                TxOut {
                    asset,
                    value: confidential::Value::Explicit(0x00000002540bd71c),
                    nonce: confidential::Nonce::Null,
                    script_pubkey: hex_script(
                        "1976a91448633e2c0ee9495dd3f9c43732c47f4702a362c888ac",
                    ),
                    witness: TxOutWitness {
                        surjection_proof: None,
                        rangeproof: None,
                    },
                },
                TxOut {
                    asset,
                    value: confidential::Value::Explicit(0x0000000000000ce4),
                    nonce: confidential::Nonce::Null,
                    script_pubkey: elements::Script::new(),
                    witness: TxOutWitness {
                        surjection_proof: None,
                        rangeproof: None,
                    },
                },
            ],
        };
        let utxo = ElementsUtxo {
            script_pubkey: elements::Script::new(),
            asset,
            value: confidential::Value::Explicit(0x00000002540be400),
        };
        let ctrl_block = ControlBlock::from_slice(&ctrl_blk).expect("ctrl block from slice");
        let script_cmr = Cmr::from(sighash_all::SIGHASH_ALL_CMR);

        ElementsEnv::new(
            Arc::new(tx),
            vec![utxo],
            0,
            script_cmr,
            ctrl_block,
            None,
            BlockHash::all_zeros(),
        )
    }
}

// Copied from rust-simplicity/src/test_progs/sighash_all.rs
mod sighash_all {
    pub const SIGHASH_ALL_CMR: [u8; 32] = [
        0xf9, 0xec, 0x96, 0x58, 0xf1, 0xa8, 0xfb, 0xd3, 0xd5, 0x6e, 0xa8, 0x3a, 0x9f, 0x23, 0xcf,
        0x3f, 0xb9, 0x12, 0x45, 0x3a, 0x3b, 0xf1, 0xdb, 0xb3, 0x2d, 0x3, 0x87, 0x66, 0x6b, 0xeb,
        0xc0, 0x1f,
    ];
}

pub trait JetEnvironment {
    type CJetEnvironment;

    fn as_ffi(&self) -> &Self::CJetEnvironment;
}

impl JetEnvironment for () {
    type CJetEnvironment = ();

    fn as_ffi(&self) -> &Self::CJetEnvironment {
        &()
    }
}

impl JetEnvironment for ElementsEnv {
    type CJetEnvironment = CElementsTxEnv;

    fn as_ffi(&self) -> &Self::CJetEnvironment {
        self.c_tx_env()
    }
}
