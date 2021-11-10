#![cfg(test)]

use crate::mock::*;
use frame_support::assert_ok;
use frame_support::sp_runtime::app_crypto::Ss58Codec;
use hex_literal::hex;
use sp_core::H256;

#[test]
fn one_level_merkel_tree_proof_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(MdPallet::create_merkle_dispatcher(
            Origin::root(),
            H256::from(&hex!(
                "056980ee78588f3d5ceab5645b2dc2838c19f938151bc1c70547664c6bf57932"
            )),
            Vec::from("test"),
            CURRENCY_TEST1,
            100 * UNIT,
        ));

        let mut proof = Vec::<H256>::new();
        proof.push(H256::from(&hex!(
            "5d6763b1aaa996a5854b019d1bd087543a1c5977d0d8c448380ca6b953007b78"
        )));
        println!("{}", BOB.to_ss58check());
        assert_ok!(MdPallet::claim(
            Origin::signed(ALICE),
            0,
            0,
            BOB,
            10_000_000_000_000_000_000,
            proof
        ));
    })
}
