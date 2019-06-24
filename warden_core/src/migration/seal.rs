use crate::migration::algo::Algo;
use crate::time;
use failure::Error;
use path_abs::{PathArc, PathFile};
use yamlette::model::schema::yamlette::Yamlette;
use yamlette::model::style::ISSUE_TAG;
use yamlette::model::yaml::binary::BinaryValue;
use yamlette::model::yaml::str::FORCE_QUOTES;
use yamlette::model::{DateTime, Fraction};
use yamlette::yamlette;

#[derive(Debug, Clone)]
pub struct Seal {
    pub timestamp: DateTime,
    pub algo: Algo,
    pub sign: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SealMeta {
    file: PathArc,
    algo: Algo,
}

impl SealMeta {
    pub fn build(file: PathArc, algo: Algo) -> Self {
        Self {
            file: file,
            algo: algo,
        }
    }

    pub fn get_algo(&self) -> Algo {
        self.algo
    }

    pub fn get_file(&self) -> &PathArc {
        &self.file
    }

    pub fn make(&self, content: &[u8]) -> Result<(), Error> {
        let hash = self.get_algo().hash(content);
        Ok(PathFile::create(self.get_file())?.write_str(&self.yamlette(hash)?)?)
    }

    fn yamlette(&self, hash: Vec<u8>) -> Result<String, Error> {
        let version = Fraction::new(1u8, 10u8);
        let schema = Yamlette::new();

        let algo = self.get_algo().stringify();
        let hash = BinaryValue::from(hash);
        let time = time::yamlette_timestamp_value();

        Ok(yamlette!(
            write ;
            [
                [ { "version": version } ],
                [ {
                    "timestamp": (# ISSUE_TAG => time),
                    "seal": {
                        "algo": (# FORCE_QUOTES => algo),
                        "sign": (# ISSUE_TAG => hash)
                    }
                } ]
            ]
            ; { schema: schema }
        )?)
    }

    pub fn read_the_seal_v_0_1(file: PathFile) -> Result<Seal, Error> {
        let mut file_read = file.read()?;
        let schema = Yamlette::new();
        yamlette!(
            read ;
            &mut file_read ;
            [[], [{
                "timestamp" => (timestamp:DateTime),
                "seal" => {
                    "algo" => (algo:String),
                    "sign" => (sign:Vec<u8>)
                }
            }]] ;
            { schema: schema }
        );

        let timestamp = if let Some(ts) = timestamp {
            ts
        } else {
            Err(failure::err_msg("timestamp is not defined"))?
        };

        let algo = if let Some(algo) = algo {
            Algo::from_str(&algo).ok_or(failure::err_msg(format!("unknown algo {}", algo)))?
        } else {
            Err(failure::err_msg("algo is not defined"))?
        };

        let sign = if let Some(sign) = sign {
            sign
        } else {
            Err(failure::err_msg("algo is not defined"))?
        };

        Ok(Seal {
            timestamp,
            algo,
            sign,
        })
    }

    pub fn read_the_seal(&self) -> Result<Seal, Error> {
        let file = PathFile::new(&self.file)?;
        let mut file_read = file.read()?;
        let schema = Yamlette::new();

        yamlette!(
            read ;
            &mut file_read ;
            [[{
                "version" => (version:Fraction)
            }]] ;
            { schema: schema }
        );

        if let Some(version) = version {
            if version == Fraction::new(1u8, 10u8) {
                Self::read_the_seal_v_0_1(file)
            } else {
                log::error!("Unsupported seal version {:.8}", version);
                Err(failure::err_msg("Unsupported version"))?
            }
        } else {
            log::error!("Seal version is not defined");
            Err(failure::err_msg("Unsupported version"))?
        }
    }
}
