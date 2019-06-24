use crate::migration::{meta::Meta as MigrationMeta, weight};
use crate::path;
use failure::Error;
use fraction::{BigFraction, Zero};
use path_abs::{PathDir, PathFile};
use std::str::FromStr;

use yamlette::model::schema::yamlette::Yamlette;
use yamlette::model::Fraction;
use yamlette::yamlette;

/// Raw meta contains parsed but unprocessed data
#[derive(Clone, Debug)]
pub struct RawMeta {
    path: String,
    requirements: Vec<String>,
    weight: BigFraction,
}

impl RawMeta {
    pub fn from_file(
        migration_meta: &MigrationMeta,
        base: &PathDir,
        file: &PathFile,
    ) -> Result<RawMeta, Error> {
        let yaml: String = {
            let content = file.read_string()?;
            content.parse::<PatchYamlHead>()?.into()
        };

        parse_meta(migration_meta, base, file, &yaml)
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn add_requirement(&mut self, req: String) {
        self.requirements.push(req);
    }

    pub fn get_requirements(&self) -> &[String] {
        &self.requirements
    }

    pub fn get_weight(&self) -> &BigFraction {
        &self.weight
    }
}

struct PatchYamlHead(String);

impl From<PatchYamlHead> for String {
    fn from(src: PatchYamlHead) -> String {
        src.0
    }
}

impl FromStr for PatchYamlHead {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("-- ---") {
            return Ok(Self(String::with_capacity(0)));
        }

        let mut result = String::with_capacity(s.len());

        for line in s.lines() {
            if line.starts_with("-- ") {
                result.push_str(&line[3..]);
                result.push('\n');
            } else if line.len() > 0 {
                break;
            }
        }

        Ok(Self(result))
    }
}

fn parse_meta(
    migration_meta: &MigrationMeta,
    base: &PathDir,
    file: &PathFile,
    content: &str,
) -> Result<RawMeta, Error> {
    let path = &path::relpath_to_base(base, file)[..];

    let path = String::from(if path.ends_with(".sql") {
        &path[0..path.len() - 4]
    } else {
        path
    });

    let weight = weight::alphabet(&path)?;

    if content.len() == 0 {
        return Ok(RawMeta {
            path,
            requirements: Vec::new(),
            weight,
        });
    }

    let schema = Yamlette::new();

    yamlette!(
        read ; String::from(content) ;
        [[{
            "version" => (version:Fraction)
        }]] ;
        { schema: schema }
    );

    let version = if let Some(version) = version {
        version
    } else {
        migration_meta.get_yaml_format_version().clone()
    };

    if version == Fraction::new(1u8, 10u8) {
        let (requirements, add_weight) = parse_meta_v_0_1(content)?;
        Ok(RawMeta {
            path,
            requirements,
            weight: (weight + add_weight),
        })
    } else {
        log::error!("Unsupported config version {:.8}", version);
        Err(failure::err_msg("Unsupported version"))?
    }

    // if let Some(version) = version {
    //
    // } else {
    //
    //     log::error!("Config version is not defined");
    //     Err(failure::err_msg("Unsupported version"))?
    // }
}

fn parse_meta_v_0_1(content: &str) -> Result<(Vec<String>, BigFraction), Error> {
    let schema = Yamlette::new();

    yamlette!(
        read ;
        String::from(content) ;
        [[{
            "require" => (req:String),
            "require" => (list reqs:Vec<String>),
            "weight" => (weight: BigFraction)
        }]] ;
        { schema: schema }
    );

    let requirements = (|| {
        if let Some(reqs) = reqs {
            if reqs.len() > 0 {
                return reqs;
            }
        }

        if let Some(req) = req {
            return vec![req];
        }

        Vec::new()
    })();

    let weight = if let Some(w) = weight {
        w
    } else {
        BigFraction::zero()
    };

    Ok((requirements, weight))
}
