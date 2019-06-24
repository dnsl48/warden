use failure::Error;
use path_abs::PathFile;
use std::str::FromStr;
use yamlette::yamlette;
use yamlette::model::Fraction;
use yamlette::model::schema::yamlette::Yamlette;


struct PatchYamlHead(String);

impl From<PatchYamlHead> for String {
    fn from(src: PatchYamlHead) -> String {
        src.0
    }
}

impl FromStr for PatchYamlHead {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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


pub fn requirements(src: &PathFile) -> Result<Vec<PathFile>, Error> {
    let mut content = src.read_string()?;

    if !content.starts_with("-- ---") {
        return Ok(Vec::new());
    }

    let head: String = content.parse::<PatchYamlHead>()?.into();

    requirements_version(&head)
}

fn requirements_version(content: &str) -> Result<Vec<String>, Error> {
    let schema = Yamlette::new();

    yamlette!(
        read ; String::from(content) ;
        [[{
            "version" => (version:Fraction)
        }]] ;
        { schema: schema }
    );

    if let Some(version) = version {
        if version == Fraction::new(1u8, 10u8) {
            requirements_v_0_1(content)
        } else {
            log::error!("Unsupported config version {:.8}", version);
            Err(failure::err_msg("Unsupported version"))?
        }
    } else {
        log::error!("Config version is not defined");
        Err(failure::err_msg("Unsupported version"))?
    }
}

fn requirements_v_0_1(content: &str) -> Result<Vec<String>, Error> {
    let schema = Yamlette::new();

    yamlette!(
        read ;
        String::from(content) ;
        [[], [{
            "require" => (req:String),
            "require" => (list reqs:Vec<String>)
        }]] ;
        { schema: schema }
    );

    if let Some(reqs) = reqs {
        if reqs.len() > 0 {
            return Ok(reqs)
        }
    }

    if let Some(req) = req {
        return Ok(vec![req])
    }

    Ok(Vec::new())
}
