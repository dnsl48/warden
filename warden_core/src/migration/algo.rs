use blake2::Digest;

#[derive(Copy, Clone, Debug)]
pub enum Algo {
    Blake2b,
    SHA3_512,
    SHA3_224,
}

impl Algo {
    pub fn stringify(self) -> &'static str {
        match self {
            Algo::Blake2b => "blake2b",
            Algo::SHA3_512 => "sha3-512",
            Algo::SHA3_224 => "sha3-224",
        }
    }

    pub fn from_str(string: &str) -> Option<Algo> {
        match string {
            "blake2b" => Some(Algo::Blake2b),
            "sha3-512" => Some(Algo::SHA3_512),
            "sha3-224" => Some(Algo::SHA3_224),
            _ => None,
        }
    }

    pub fn hash(self, value: &[u8]) -> Vec<u8> {
        match self {
            Algo::Blake2b => Vec::from(&blake2::Blake2b::digest(value)[..]),
            Algo::SHA3_512 => Vec::from(&sha3::Sha3_512::digest(value)[..]),
            Algo::SHA3_224 => Vec::from(&sha3::Sha3_224::digest(value)[..]),
        }
    }
}

impl Default for Algo {
    fn default() -> Algo {
        Algo::Blake2b
    }
}
