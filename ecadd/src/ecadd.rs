// bn128 point addition copied from https://github.com/ethereumjs/rustbn.js
extern crate ewasm_api;
extern crate bn;

use std::io::{self, Read};

pub struct Error(pub &'static str);

impl From<&'static str> for Error {
    fn from(val: &'static str) -> Self {
	Error(val)
    }
}

fn read_point(reader: &mut io::Chain<&[u8], io::Repeat>) -> Result<::bn::G1, Error> {
    use bn::{Fq, AffineG1, G1, Group};

    let mut buf = [0u8; 32];

    reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
    let px = Fq::from_slice(&buf[0..32]).map_err(|_| Error::from("Invalid point x coordinate"))?;

    reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
    let py = Fq::from_slice(&buf[0..32]).map_err(|_| Error::from("Invalid point y coordinate"))?;

    Ok(
	if px == Fq::zero() && py == Fq::zero() {
	    G1::zero()
	} else {
	    AffineG1::new(px, py).map_err(|_| Error::from("Invalid curve point"))?.into()
	}
    )
}

#[no_mangle]
pub fn main() {
	use bn::AffineG1;

    // try input from https://github.com/ethereum/tests/blob/9741ed0bc1fb660c5ffefd751c24bc739104ce5e/src/GeneralStateTestsFiller/stZeroKnowledge/pointAddFiller.json#L179
    // output should be https://github.com/ethereum/tests/blob/9741ed0bc1fb660c5ffefd751c24bc739104ce5e/src/GeneralStateTestsFiller/stZeroKnowledge/pointAddFiller.json#L40

    let input_offset: usize = 0;
    let input_length: usize = 128;

    let ewasm_input:Vec<u8> = ewasm_api::calldata_copy(input_offset, input_length);    

    let mut padded_input = ewasm_input.chain(io::repeat(0));

    let mut padded_buf = [0u8; 128];
    padded_input.read_exact(&mut padded_buf[..]).expect("reading from zero-extended memory cannot fail; qed");

    let point1 = &padded_buf[0..64];
    let point2 = &padded_buf[64..128];

    let mut point1_padded = point1.chain(io::repeat(0));
    let mut point2_padded = point2.chain(io::repeat(0));

    let p1;
    match read_point(&mut point1_padded) {
        Ok(p) => {
            p1 = p;
        },
        Err(_) => { return; }
    }

    match read_point(&mut point2_padded) {
        Ok(p) => {
            let p2 = p;
            let mut ecadd_output_buf = [0u8; 64];
            if let Some(sum) = AffineG1:: from_jacobian(p1 + p2) {
                sum.x().to_big_endian(&mut ecadd_output_buf[0..32]).expect("Cannot fail since 0..32 is 32-byte length");
                sum.y().to_big_endian(&mut ecadd_output_buf[32..64]).expect("Cannot fail since 32..64 is 32-byte length");
            }


            let vec_buf = ecadd_output_buf.to_vec();

            ewasm_api::finish_data(vec_buf);

            return;
        },
        Err(_) => { return; }
    }

}
