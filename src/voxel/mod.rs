use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::util::display_iter::DisplayableExt;

pub mod material;
pub mod model_storage;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelEntry(pub Model);

impl Display for ModelEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Model {
    Min(Box<[Model]>),

    AABB { min: [f32; 3], max: [f32; 3] },
    Sphere { position: [f32; 3], radius: f32 },
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Model::Min(els) => {
                let i = els.iter().into_displayable(',');
                f.write_fmt(format_args!("min({})", i))
            }
            Model::AABB { min, max } => f.write_fmt(format_args!(
                "aabb(p, vec3<f32>({:.9}), vec3<f32>({:.9}))",
                min.iter().into_displayable(','),
                max.iter().into_displayable(',')
            )),
            Model::Sphere { position, radius } => f.write_fmt(format_args!(
                "sphere(p, vec3<f32>({:.9}), {:.9})",
                position.iter().into_displayable(','),
                radius
            )),
        }
    }
}
