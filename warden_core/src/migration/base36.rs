pub fn encode(mut value: u128) -> String {
    let mut idx = 25;
    let mut buffer: [u8; 25] = [0; 25];

    while value > 0 {
        let key: u8 = (value % 36u128) as u8;
        idx -= 1;

        buffer[idx as usize] = match key {
            0 ... 9 => b'0' + key,
            10 ... 35 => b'a' + (key - 10),
            _ => unreachable!()
        };

        value = value / 36u128;
    }

    unsafe { String::from_utf8_unchecked(buffer[25 - (25 - idx) as usize .. 25].to_vec()) }
}


pub fn decode(value: &str) -> Option<u128> {
    u128::from_str_radix(value, 36).ok()
}


#[cfg(test)]
mod tests {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    use super::*;

    #[test]
    fn test_integrity() {
        for _ in 0..10000 {
            let mut hasher = RandomState::new().build_hasher();
            hasher.write_u32(1);
            let value = hasher.finish() as u128;

            let e = encode(value);
            let d = decode(&e);

            assert_eq!(d, Some(value));
        }

    }
}