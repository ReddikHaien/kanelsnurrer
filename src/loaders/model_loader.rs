use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    error::Error,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use bevy::{
    prelude::{Assets, In, ResMut, Shader},
    utils::{tracing::callsite::Identifier, HashSet},
};
use serde::Deserialize;

use crate::{
    voxel::{model_storage::ModelStorage, Model, ModelEntry},
    world::tile::material_identifier::MaterialIdentifierElement,
};

use super::LoadingInfo;

#[derive(Deserialize)]
enum ModelElement {
    Min {
        elements: Box<[ModelElement]>,
        texture: Option<String>,
    },
    AABB {
        min: [f32; 3],
        max: [f32; 3],
        texture: Option<String>,
    },
    Sphere {
        position: [f32; 3],
        radius: f32,
        texture: Option<String>,
    },
}

impl Into<Model> for ModelElement {
    fn into(self) -> Model {
        match self {
            ModelElement::Min { elements, texture } => {
                let data = elements.into_vec().into_iter().map(|x| x.into()).collect();
                Model::Min(data)
            }
            ModelElement::AABB { min, max, texture } => Model::AABB { min, max },
            ModelElement::Sphere {
                position,
                radius,
                texture,
            } => Model::Sphere { position, radius },
        }
    }
}

impl ModelElement {
    pub fn for_each_texture<F>(&self, mut cb: F)
    where
        F: FnMut(&str),
    {
        let mut stack = VecDeque::new();
        stack.push_back(self);

        while let Some(x) = stack.pop_front() {
            match x {
                ModelElement::Min { elements, texture } => {
                    if let Some(t) = texture {
                        cb(t)
                    }
                    for y in elements.iter() {
                        stack.push_back(y)
                    }
                }
                ModelElement::AABB { texture, .. } | ModelElement::Sphere { texture, .. } => {
                    if let Some(t) = texture {
                        cb(t)
                    }
                }
            }
        }
    }
}

struct StorageBuilder {
    models: Vec<(Vec<MaterialIdentifierElement>, ModelElement)>,
    textures: HashSet<String>,
}

impl StorageBuilder {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            textures: HashSet::new(),
        }
    }

    pub fn add_new(&mut self, identifier: Vec<MaterialIdentifierElement>, model: ModelElement) {
        model.for_each_texture(|s| {
            self.textures.insert(s.to_string());
        });
        self.models.push((identifier, model));
    }

    pub fn print_info(&self) {
        println!(
            "{} textures and {} models",
            self.textures.len(),
            self.models.len()
        );
    }
}

pub fn wait_textures() {}

pub(super) fn load_models(
    mut storage: ResMut<ModelStorage>,
    shaders: ResMut<Assets<Shader>>,
    mut info: ResMut<LoadingInfo>,
) {
    let mut builder = StorageBuilder::new();

    read_dir(
        &PathBuf::from_str("assets/models").unwrap(),
        &mut Vec::new(),
        &mut builder,
    )
    .unwrap();

    builder.print_info();

    //======= Image loading

    //=====================

    for (name, model) in builder.models {
        storage.add_model(ModelEntry(model.into()), &name);
    }

    storage.generate_shader_assets(shaders);

    info.loaded += 1;
}

fn read_dir(
    path: &Path,
    identifier: &mut Vec<MaterialIdentifierElement>,
    storage: &mut StorageBuilder,
) -> Result<(), Box<dyn Error>> {
    let dir = fs::read_dir(path)?;

    for x in dir {
        let x = x?;
        let file_type = x.file_type()?;

        if file_type.is_dir() {
            identifier.push(x.file_name().to_str().unwrap().to_uppercase().into());
            read_dir(&x.path(), identifier, storage)?;
            identifier.pop();
        } else {
            if !x.file_name().to_str().unwrap().starts_with("mod") {
                let ol = x.file_name();
                let l = ol.to_str().unwrap();
                identifier.push(l[0..(l.len() - 4)].to_uppercase().into());
                read_file(&x.path(), identifier, storage)?;
                identifier.pop();
            } else {
                read_file(&x.path(), identifier, storage)?;
            }
        }
    }

    Ok(())
}

fn read_file(
    path: &Path,
    identifier: &mut Vec<MaterialIdentifierElement>,
    storage: &mut StorageBuilder,
) -> Result<(), Box<dyn Error>> {
    println!("loading {:?}", path);

    let bytes = fs::read(path)?;

    let source = String::from_utf8(bytes)?;

    let model: ModelElement = ron::from_str(&source)?;

    storage.add_new(identifier.clone(), model);

    Ok(())
}
