#![cfg(test)]

use crate::mock::*;
use frame_support::assert_ok;
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

#[test]
fn set_claimed_should_work() {
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

        assert_ok!(MdPallet::create_merkle_dispatcher(
            Origin::root(),
            H256::from(&hex!(
                "056980ee78588f3d5ceab5645b2dc2838c19f938151bc1c70547664c6bf57932"
            )),
            Vec::from("test2"),
            CURRENCY_TEST1,
            100 * UNIT,
        ));

        for i in 0u32..20000 {
            MdPallet::set_claimed(0, i);
            MdPallet::set_claimed(1, i);
        }

        for i in 0u32..20000 {
            assert!(MdPallet::is_claimed(0, i));
            assert!(MdPallet::is_claimed(1, i));
        }
    })
}

#[test]
fn set_claimed_should_not_work() {
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

        assert_ok!(MdPallet::create_merkle_dispatcher(
            Origin::root(),
            H256::from(&hex!(
                "056980ee78588f3d5ceab5645b2dc2838c19f938151bc1c70547664c6bf57932"
            )),
            Vec::from("test2"),
            CURRENCY_TEST1,
            100 * UNIT,
        ));

        for i in 0u32..2000 {
            MdPallet::set_claimed(0, i);
            MdPallet::set_claimed(1, i);
        }

        for i in 2000u32..20000 {
            assert_eq!(MdPallet::is_claimed(0, i), false);
            assert_eq!(MdPallet::is_claimed(1, i), false);
        }
    })
}
