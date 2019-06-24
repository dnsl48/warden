use super::algo::Algo;
use super::identity::Identity;
use super::path::FileOrDir;
use super::seal::SealMeta;
use crate::path;
use failure::{self, Error};
use path_abs::{PathArc, PathDir, PathFile};

use yamlette::model::schema::yamlette::Yamlette;
use yamlette::model::yaml::str::FORCE_QUOTES;
use yamlette::model::Fraction;
use yamlette::yamlette;

#[derive(Debug, Clone)]
pub struct Meta {
    yaml_format_version: Fraction,
    path: PathFile,
    identity: Identity,
    seal_meta: SealMeta,
    source: FileOrDir,
    target: PathArc,
}

impl Meta {
    pub fn default_yaml_format_version() -> Fraction {
        Fraction::new(1u8, 10u8)
    }

    pub fn get_identity(&self) -> &Identity {
        &self.identity
    }

    pub fn get_seal_meta(&self) -> &SealMeta {
        &self.seal_meta
    }

    pub fn get_source(&self) -> &FileOrDir {
        &self.source
    }

    pub fn get_target(&self) -> &PathArc {
        &self.target
    }

    pub fn get_path(&self) -> &PathFile {
        &self.path
    }

    pub fn get_source_base(&self) -> PathDir {
        self.source
            .map(
                |file| {
                    Ok(file
                        .parent_dir()
                        .expect("every file must have a parent folder"))
                },
                |dir| Ok(dir.clone()),
            )
            .unwrap()
    }

    pub fn get_base(&self) -> PathDir {
        self.path
            .parent_dir()
            .expect("every file must have a parent folder")
    }

    pub fn get_yaml_format_version(&self) -> &Fraction {
        &self.yaml_format_version
    }

    pub fn create(
        yaml_format_version: Fraction,
        path: PathFile,
        id: Identity,
        seal_meta: SealMeta,
        source: FileOrDir,
        target: PathArc,
    ) -> Result<Meta, Error> {
        let meta = Self {
            yaml_format_version: yaml_format_version,
            path: path,
            identity: id,
            seal_meta: seal_meta,
            source: source,
            target: target,
        };
        meta.save()
    }

    pub fn save(self) -> Result<Meta, Error> {
        self.path.write_str(&self.yamlette()?)?;
        Ok(self)
    }

    pub fn open(file: PathFile) -> Result<Meta, Error> {
        Self::parse_meta(file)
    }

    pub fn yamlette(&self) -> Result<String, Error> {
        // let syntax_version = Fraction::new(1u8, 10u8);
        let schema = Yamlette::new();

        let root = self.get_base();

        let syntax_version = self.get_yaml_format_version().clone();
        let uid = self.identity.get_uid().to_string();
        let name = self.identity.get_name().to_string();
        let source = path::relpath_to_base(&root, &self.source.get_path()).replace("./", "");
        let target = path::relpath_to_base(&root, &self.target).replace("./", "");

        let seal_file = path::relpath_to_base(&root, self.seal_meta.get_file()).replace("./", "");
        let seal_algo = self.seal_meta.get_algo().stringify();

        Ok(yamlette!(
            write ;
            [
            [ { "version": syntax_version } ],
            [ {
                "identity": {
                    "uid": (# FORCE_QUOTES => uid),
                    "name": (# FORCE_QUOTES => name)
                },

                "structure": {
                    "source": (# FORCE_QUOTES => source),
                    "target": (# FORCE_QUOTES => target)
                },

                "seal": {
                    "file": (# FORCE_QUOTES => seal_file),
                    "algo": (# FORCE_QUOTES => seal_algo)
                }
            } ]
            ]
            ; { schema: schema }
        )?)
    }

    fn parse_meta(file: PathFile) -> Result<Meta, Error> {
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
                Self::parse_meta_v_0_1(file)
            } else {
                log::error!("Unsupported meta version {:.8}", version);
                Err(failure::err_msg("Unsupported version"))?
            }
        } else {
            log::error!("Meta version is not defined");
            Err(failure::err_msg("Unsupported version"))?
        }
    }

    fn parse_meta_v_0_1(file: PathFile) -> Result<Meta, Error> {
        let root = file
            .parent_dir()
            .ok_or(failure::err_msg("Could not fetch the meta file parent dir"))?;

        let mut file_read = file.read()?;
        let schema = Yamlette::new();
        yamlette!(
            read ;
            &mut file_read ;
            [[], [ {
                "identity" => {
                    "uid" => (uid:String),
                    "name" => (name:String)
                },

                "structure" => {
                    "source" => (source:String),
                    "target" => (target:String)
                },

                "seal" => {
                    "file" => (seal_file:String),
                    "algo" => (seal_algo:String)
                }
            } ]] ;
            { schema: schema }
        );

        let identity = Identity::build(
            uid.ok_or(failure::err_msg("identity.uid is undefined"))?,
            name.ok_or(failure::err_msg("identity.name is undefined"))?,
        );

        let seal_algo = seal_algo.ok_or(failure::err_msg("seal.algo is undefined"))?;
        let seal_meta = SealMeta::build(
            root.join(seal_file.ok_or(failure::err_msg("seal.file is undefined"))?),
            Algo::from_str(&seal_algo).ok_or(failure::err_msg(format!(
                "unsupported seal.algo: \"{}\"",
                seal_algo
            )))?,
        );

        let source_path = root
            .join(source.ok_or(failure::err_msg("structure.source is undefined"))?)
            .absolute()?;

        let source_path = if let Ok(source_path) = PathFile::new(&source_path) {
            Ok(FileOrDir::from(source_path))
        } else if let Ok(source_path) = PathDir::new(&source_path) {
            Ok(FileOrDir::from(source_path))
        } else {
            Err(failure::err_msg(format!(
                "source path does not exist: \"{:?}\"",
                source_path
            )))
        }?;

        let target_path =
            root.join(target.ok_or(failure::err_msg("structure.target is undefined"))?);

        Self::create(
            Fraction::new(1u8, 10u8),
            file,
            identity,
            seal_meta,
            source_path,
            target_path,
        )
    }
}
