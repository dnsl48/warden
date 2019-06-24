use crate::dbms::driver;
use failure::Error;
use log;
use yamlette::yamlette;
// use yamlette::orchestra::OrchError;
use yamlette::model::Fraction;
use yamlette::model::schema::yamlette::Yamlette;
use yamlette::model::yamlette::env;
use yamlette::model::yamlette::incognitum::IncognitumValue;
use yamlette::model::yaml::str::FORCE_QUOTES;
use std::borrow::Cow;

pub fn migrations_relpath() -> Cow<'static, str> {
    Cow::from("../migrations")
}

pub fn generate_initial(driver: &str) -> Result<String, Error> {
    log::trace!("Validating driver: {}", driver);
    if !validate_driver(driver) {
        log::error!("Invalid dbms driver: {}", driver);
        Err(failure::err_msg(format!("Unknown database driver: {}", driver)))?;
    }

    let driver = String::from(driver);

    let version = Fraction::new(1u8, 10u8);
    let connection = IncognitumValue::new(Cow::from("DATABASE_URL")).set_tag(Cow::from(env::TAG));
    let schema = Yamlette::new();

    let migrations = migrations_relpath();

    Ok(yamlette!(
        write ;
        [
            [ { "version": version } ],
            [ {
                "connection": connection,
                "repository": (# FORCE_QUOTES => ".."),
                "migrations": (# FORCE_QUOTES => migrations),
                "driver": (# FORCE_QUOTES => driver)
            } ]
        ]
        ; { schema: schema }
    )?)
}

fn validate_driver(name: &str) -> bool {
    driver::drivers(|factory| {
        if factory.name() == name {
            Some(Ok(true))
        } else {
            None
        }
    })
    .ok()
    .unwrap()
    .unwrap_or(false)
}
