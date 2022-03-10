use rustbreak::{deser::Yaml, PathDatabase};
use serde::{Deserialize, Serialize};

use crate::models::AppError;

pub struct GlobalFeatures {
    features: PathDatabase<Vec<Feature>, Yaml>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Feature {
    Login,
}

impl GlobalFeatures {
    pub fn from_path(path: &str) -> Self {
        let features: PathDatabase<Vec<Feature>, Yaml> =
            PathDatabase::load_from_path_or_else(path.into(), Vec::new).unwrap();
        Self { features }
    }

    pub fn actives(&self, feature: &Feature) -> Result<bool, AppError> {
        Ok(self.features.borrow_data()?.contains(feature))
    }

    pub fn all(&self) -> Result<Vec<Feature>, AppError> {
        let features = self.features.borrow_data()?.clone();
        Ok(features)
    }

    pub fn activate(&self, feature: &Feature) -> Result<(), AppError> {
        let mut features = self.features.borrow_data_mut()?;
        if !features.contains(feature) {
            features.push(feature.clone());
        }
        Ok(())
    }

    pub fn desactivate(&self, feature: &Feature) -> Result<(), AppError> {
        let mut features = self.features.borrow_data_mut()?;
        features.retain(|f| f != feature);
        Ok(())
    }
}
