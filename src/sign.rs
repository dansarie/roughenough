// Copyright 2017 int08h LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Ed25519 signing and verification
//!
//! `Ring` does not provide a multi-step (init-update-finish) interface
//! for Ed25519 signatures. `Verifier` and `Signer` provide this 
//! missing multi-step api.

extern crate ring;
extern crate untrusted;

use self::ring::signature;
use self::ring::signature::Ed25519KeyPair;

use self::untrusted::Input;

/// A multi-step (init-update-finish) interface for verifying an 
/// Ed25519 signature
#[derive(Debug)]
pub struct Verifier<'a> {
    pubkey: Input<'a>,
    buf: Vec<u8>,
}

impl<'a> Verifier<'a> {
    pub fn new(pubkey: &'a [u8]) -> Self {
        Verifier {
            pubkey: Input::from(pubkey),
            buf: Vec::with_capacity(256),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.buf.reserve(data.len());
        self.buf.extend_from_slice(data);
    }

    pub fn verify(&self, expected_sig: &[u8]) -> bool {
        let msg = Input::from(&self.buf);
        let sig = Input::from(expected_sig);

        match signature::verify(&signature::ED25519, self.pubkey, msg, sig) {
            Ok(_) => true,
            _ => false,
        }
    }
}

/// A multi-step (init-update-finish) interface for creating an 
/// Ed25519 signature
pub struct Signer {
    key_pair: Ed25519KeyPair,
    buf: Vec<u8>,
}

impl Signer {
    pub fn new(seed: &[u8]) -> Self {
        Signer {
            key_pair: Ed25519KeyPair::from_seed_unchecked(Input::from(seed)).unwrap(),
            buf: Vec::with_capacity(256),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.buf.reserve(data.len());
        self.buf.extend_from_slice(data);
    }

    pub fn sign(&mut self) -> Vec<u8> {
        let signature = self.key_pair.sign(&self.buf).as_ref().to_vec();
        self.buf.clear();

        signature
    }

    pub fn public_key_bytes(&self) -> &[u8] {
        self.key_pair.public_key_bytes()
    }
}

#[cfg(test)]
mod test {
    use hex::*;
    use super::*;

    #[test]
    fn verify_ed25519_sig_on_empty_message() {
        let pubkey = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
            .from_hex()
            .unwrap();

        let signature = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
            .from_hex()
            .unwrap();

        let v = Verifier::new(&pubkey);
        let result = v.verify(&signature);
        assert_eq!(result, true);
    }

    #[test]
    fn verify_ed25519_sig() {
        let pubkey = "c0dac102c4533186e25dc43128472353eaabdb878b152aeb8e001f92d90233a7"
            .from_hex()
            .unwrap();

        let message = "5f4c8989".from_hex().unwrap();

        let signature = "124f6fc6b0d100842769e71bd530664d888df8507df6c56dedfdb509aeb93416e26b918d38aa06305df3095697c18b2aa832eaa52edc0ae49fbae5a85e150c07"
            .from_hex()
            .unwrap();

        let mut v = Verifier::new(&pubkey);
        v.update(&message);
        let result = v.verify(&signature);
        assert_eq!(result, true);
    }

    #[test]
    fn sign_ed25519_empty_message() {
        let seed = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
            .from_hex()
            .unwrap();

        let expected_sig = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
            .from_hex()
            .unwrap();

        let mut s = Signer::new(&seed);
        let sig = s.sign();
        assert_eq!(sig, expected_sig);
    }

    #[test]
    fn sign_ed25519_message() {
        let seed = "0d4a05b07352a5436e180356da0ae6efa0345ff7fb1572575772e8005ed978e9"
            .from_hex()
            .unwrap();

        let message = "cbc77b".from_hex().unwrap();

        let expected_sig = "d9868d52c2bebce5f3fa5a79891970f309cb6591e3e1702a70276fa97c24b3a8e58606c38c9758529da50ee31b8219cba45271c689afa60b0ea26c99db19b00c"
            .from_hex()
            .unwrap();

        let mut s = Signer::new(&seed);
        s.update(&message);
        let sig = s.sign();
        assert_eq!(sig, expected_sig);
    }

    #[test]
    fn sign_verify_round_trip() {
        let seed = "334a05b07352a5436e180356da0ae6efa0345ff7fb1572575772e8005ed978e9"
            .from_hex()
            .unwrap();

        let message = "Hello world".as_bytes();

        let mut signer = Signer::new(&seed);
        signer.update(&message);
        let signature = signer.sign();

        let mut v = Verifier::new(signer.public_key_bytes());
        v.update(&message);
        let result = v.verify(&signature);

        assert_eq!(result, true);
    }

}
