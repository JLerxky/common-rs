// Copyright Rivtower Technologies LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use color_eyre::eyre::{eyre, Result};
use efficient_sm2::{KeyPair, PublicKey, Signature};

pub const SM2_SIGNATURE_BYTES_LEN: usize = 128;
pub const SM2_PUBLIC_KEY_LEN: usize = 64;
pub const HASH_BYTES_LEN: usize = 32;
pub const ADDR_BYTES_LEN: usize = 20;

pub fn private_key_to_public_key(private_key: &[u8]) -> Result<[u8; SM2_PUBLIC_KEY_LEN]> {
    let key_pair = efficient_sm2::KeyPair::new(private_key)
        .map_err(|e| eyre!("create sm key_pair failed: {e:?}"))?;
    let mut public_key_bytes = [0u8; SM2_PUBLIC_KEY_LEN];
    public_key_bytes.copy_from_slice(&key_pair.public_key().bytes_less_safe()[1..]);
    Ok(public_key_bytes)
}

pub fn sign(pubkey: &[u8], privkey: &[u8], msg: &[u8]) -> Result<[u8; SM2_SIGNATURE_BYTES_LEN]> {
    let key_pair =
        KeyPair::new(privkey).map_err(|e| eyre!("sm sign: KeyPair_new failed: {:?}", e))?;
    let sig = key_pair
        .sign(msg)
        .map_err(|e| eyre!("sm sign: KeyPair_sign failed: {:?}", e))?;

    let mut sig_bytes = [0u8; SM2_SIGNATURE_BYTES_LEN];
    sig_bytes[..32].copy_from_slice(&sig.r());
    sig_bytes[32..64].copy_from_slice(&sig.s());
    sig_bytes[64..].copy_from_slice(pubkey);
    Ok(sig_bytes)
}

pub fn verify(address: &[u8], signature: &[u8], message: &[u8]) -> Result<()> {
    if signature.len() != SM2_SIGNATURE_BYTES_LEN {
        return Err(eyre!(
            "sm verify: signature length is not {}",
            SM2_SIGNATURE_BYTES_LEN
        ));
    }

    if address != recover(signature, message)? {
        Err(eyre!("sm verify: address is not match"))
    } else {
        Ok(())
    }
}

fn hash(input: &[u8]) -> [u8; HASH_BYTES_LEN] {
    let mut result = [0u8; HASH_BYTES_LEN];
    result.copy_from_slice(libsm::sm3::hash::Sm3Hash::new(input).get_hash().as_ref());
    result
}

pub fn pk2address(pk: &[u8]) -> [u8; ADDR_BYTES_LEN] {
    let mut result = [0u8; ADDR_BYTES_LEN];
    result.copy_from_slice(&hash(pk)[HASH_BYTES_LEN - ADDR_BYTES_LEN..]);
    result
}

fn recover(signature: &[u8], message: &[u8]) -> Result<[u8; ADDR_BYTES_LEN]> {
    let r = &signature[0..32];
    let s = &signature[32..64];
    let pk = &signature[64..];

    let public_key = PublicKey::new(&pk[..32], &pk[32..]);
    let sig =
        Signature::new(r, s).map_err(|e| eyre!("sm recover: Signature_new failed: {:?}", e))?;

    sig.verify(&public_key, message)
        .map_err(|e| eyre!("sm recover: Signature_verify failed: {:?}", e))?;

    Ok(pk2address(pk))
}
