use crate::migration::base36;

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Identity {
    uid: String,
    name: String
}

impl Identity {
    pub fn new(name: String) -> Self {
        Identity { uid: Self::generate_uid(), name: name }
    }

    pub fn build(uid: String, name: String) -> Self {
        Identity {
            uid: uid,
            name: name
        }
    }

    pub fn from_str(token: &str) -> Option<Identity> {
        if let Some(ix) = token.find("--") {
            let (uid, name) = token.split_at(ix);

            Some(Identity::build(String::from(uid), String::from(&name[2..])))
        } else {
            None
        }
    }

    pub fn get_uid(&self) -> &str {
        &self.uid
    }

    pub fn get_id(&self) -> Option<u128> {
        base36::decode(&self.uid)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn generate_uid() -> String {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
        let sec = ts.as_secs() - 49 * 365 * 24 * 3600;
        format!("{:0>6}", base36::encode(sec as u128))
    }
}


impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}--{}", self.get_uid(), self.get_name())
    }
}
