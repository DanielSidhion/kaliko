use sha2::{Digest, Sha256};

static BASE58_CHARACTERS: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
static BASE58_MAP: [Option<u8>; 256] = [
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    Some(0), Some(1), Some(2), Some(3), Some(4), Some(5), Some(6),
    Some(7), Some(8), None,    None,    None,    None,    None,    None,
    None,    Some(9), Some(10),Some(11),Some(12),Some(13),Some(14),Some(15),
    Some(16),None,    Some(17),Some(18),Some(19),Some(20),Some(21),None,
    Some(22),Some(23),Some(24),Some(25),Some(26),Some(27),Some(28),Some(29),
    Some(30),Some(31),Some(32),None,    None,    None,    None,    None,
    None,    Some(33),Some(34),Some(35),Some(36),Some(37),Some(38),Some(39),
    Some(40),Some(41),Some(42),Some(43),None,    Some(44),Some(45),Some(46),
    Some(47),Some(48),Some(49),Some(50),Some(51),Some(52),Some(53),Some(54),
    Some(55),Some(56),Some(57),None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
    None,    None,    None,    None,    None,    None,    None,    None,
];

pub enum Error {
    InvalidByte,
    WrongChecksum,
}

pub fn encode<I>(data: I) -> String
    where
        I: Iterator<Item = u8>
{
    let (len, _) = data.size_hint();
    let result_length = 1 + len * 138 / 100;
    let mut result_numbers = vec![0; result_length];

    let mut leading_zeroes = 0;
    let mut looking_for_zeroes = true;
    let mut actual_length = 0;

    for byte in data {
        let mut carry = byte as usize;

        if looking_for_zeroes && carry == 0 {
            leading_zeroes += 1;
            continue
        }

        looking_for_zeroes = false;

        let mut i = 0;

        while (carry != 0 || i < actual_length) && i < result_length {
            let curr_total = (result_numbers[i] as usize) * 256 + carry;
            result_numbers[i] = (curr_total % 58) as u8;
            carry = curr_total / 58;
            i += 1;
        }

        actual_length = i;
    }

    result_numbers.truncate(actual_length);

    // '1' is the representation for 0 bytes, so pushing 0 will work here since BASE58_CHARACTERS[0] == '1'.
    for _ in 0..leading_zeroes {
        result_numbers.push(0);
    }

    result_numbers.reverse();

    for num in result_numbers.iter_mut() {
        *num = BASE58_CHARACTERS[*num as usize];
    }

    String::from_utf8(result_numbers).unwrap()
}

pub fn check_encode(data: &[u8]) -> String {
    let checksum = Sha256::digest(Sha256::digest(data).as_slice());

    encode(data.iter().cloned().chain(checksum[0..4].iter().cloned()))
}

pub fn decode(data: &str) -> Result<Vec<u8>, Error> {
    let len = 1 + data.len() * 733 / 1000;
    let mut result = vec![0u8; len];

    let mut actual_length = 0;
    let mut leading_zeroes = 0;
    let mut looking_for_zeroes = true;

    for byte in data.bytes() {
        let mut carry = match BASE58_MAP[byte as usize] {
            Some(val) => val as usize,
            None => return Err(Error::InvalidByte),
        };

        if looking_for_zeroes && carry == 0 {
            leading_zeroes += 1;
            continue;
        }

        looking_for_zeroes = false;

        let mut i = 0;
        while (carry != 0 || i < actual_length) && i < len {
            let curr_total = (result[i] as usize) * 58 + carry;
            result[i] = (curr_total % 256) as u8;
            carry = curr_total / 256;
            i += 1;
        }

        actual_length = i;
    }

    result.truncate(actual_length);

    for _ in 0..leading_zeroes {
        result.push(0u8);
    }

    result.reverse();

    Ok(result)
}

pub fn decode_check(data: &str) -> Result<Vec<u8>, Error> {
    let mut decoded_data = decode(data)?;
    let decoded_len = decoded_data.len();
    let checksum = decoded_data.split_off(decoded_len - 4);

    let double_sha256_digest = Sha256::digest(&Sha256::digest(&decoded_data));

    if double_sha256_digest.starts_with(&checksum) {
        return Ok(decoded_data)
    }

    Err(Error::WrongChecksum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::{FromHex};

    #[test]
    fn decode_works() {
        assert_eq!(decode("1").ok(), Some(vec![0u8]));
        assert_eq!(decode("2").ok(), Some(vec![1u8]));
        assert_eq!(decode_check("1PfJpZsjreyVrqeoAfabrRwwjQyoSQMmHH").ok(), Some(Vec::from_hex("00f8917303bfa8ef24f292e8fa1419b20460ba064d").unwrap()));
    }
}