use ark_bls12_381::{Fq, Fr};
use ark_ec::AffineCurve;
use ark_ff::biginteger::BigInteger256;
use prompt::{puzzle, welcome};
use std::str::FromStr;
use trusted_setup::data::puzzle_data;
use trusted_setup::PUZZLE_DESCRIPTION;

fn main() {
    welcome();
    puzzle(PUZZLE_DESCRIPTION);
    let (_ts1, _ts2) = puzzle_data();

    /* Your solution here! (s in decimal)*/

    let s = Fr::from_str("114939083266787167213538091034071020048").unwrap();

    // println!("{}", _ts2[1]);

    assert_eq!(_ts1[0].mul(s), _ts1[1]);
    assert_eq!(_ts2[0].mul(s), _ts2[1]);
}
