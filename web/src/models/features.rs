use rustbreak::{deser::Yaml, PathDatabase};
use serde::{Deserialize, Serialize};

use crate::models::AppError;

pub struct GlobalFeatures {
    features: PathDatabase<Features, Yaml>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Features {
    pub login: LoginFeature,
}

impl Default for Features {
    fn default() -> Self {
        Features {
            login: LoginFeature {
                simple: false,
                read_private: false,
                write_private: false,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LoginFeature {
    pub simple: bool,
    pub read_private: bool,
    pub write_private: bool,
}

impl GlobalFeatures {
    pub fn from_path(path: &str) -> Self {
        let features: PathDatabase<Features, Yaml> =
            PathDatabase::load_from_path_or_else(path.into(), Features::default).unwrap();
        Self { features }
    }

    pub fn get(&self) -> Result<Features, AppError> {
        let features = self.features.borrow_data()?.clone();
        Ok(features)
    }

    pub fn patch(&self, new_features: &PatchableFeatures) -> Result<(), AppError> {
        {
            let mut features = self.features.borrow_data_mut()?;

            if let Some(login) = &new_features.login {
                if let Some(simple) = login.simple {
                    features.login.simple = simple;
                }
            }
        }
        self.features.save()?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PatchableFeatures {
    login: Option<PatchableLoginFeature>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PatchableLoginFeature {
    simple: Option<bool>,
    read_private: Option<bool>,
    write_private: Option<bool>,
}
