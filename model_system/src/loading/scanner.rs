use std::{path::{Path, PathBuf}, fs, f32::consts::E, error::Error, io, collections::BTreeMap};

use crate::{format::ModelFile, naming::{Element, MaterialIdentifier}};

use super::model_files::LoadedModels;

pub struct Scanner{
    files: Vec<(MaterialIdentifier, String)>,
    name: Vec<Element>, 
}

impl Scanner{
    pub fn new() -> Self{
        Self{
            files: Vec::new(),
            name: Vec::new(),
        }
    }

    pub fn scan_dir(&mut self, dir: &Path) -> Result<(), io::Error>{
        println!("loading: {:?}",dir);
        let dir = fs::read_dir(dir)?;

        for x in dir{
            let entry = x?;
            let file_type = entry.file_type()?;
            if file_type.is_dir(){
                self.name.push(Element::Custom(entry.file_name().to_str().unwrap().to_owned()));
                self.scan_dir(&entry.path())?;
                self.name.pop();
            }
            else{

                let file_name = entry.file_name().to_str().unwrap().to_owned();
                let path = entry.path();
                let content = fs::read_to_string(&path)?;

                if file_name.as_str() == "mod.json"{
                    self.files.push((MaterialIdentifier::clone_new_from(&self.name), content));
                }
                else{
                    self.files.push((MaterialIdentifier::clone_new_from(&self.name), content));
                }
            }
        }
        Ok(())
    }

    pub fn load<'a>(&'a self) -> Result<LoadedModels<'a>, Box<dyn Error>>{

        let mut out = BTreeMap::new();

        for (path,content ) in &self.files{
            let json = serde_json::de::from_str::<ModelFile>(content)?;
            out.insert(path, json);
        }
        Ok(LoadedModels::new(out))
    }
}

mod test{
    use std::path::PathBuf;

    use crate::format::{ModelFile, ShapeKinds, ShapeEntry, VisibilityDefinition, Visibility};

    use super::Scanner;

    #[test]
    fn load_models(){
        let mut scanner = Scanner::new();
        scanner.scan_dir(&PathBuf::from("./test_files")).unwrap();
        let models = scanner.load().unwrap();
        assert!(models.num_entries() == 1);
        let expected_model = ModelFile{
            inherits: None,
            shape_kinds: ShapeKinds{
                wall: Some(ShapeEntry{
                    model: vec![

                    ],
                    visibility: VisibilityDefinition{
                        all: Some(Visibility::Solid),
                        ..Default::default()
                    }
                }),
                floor: None
            }
        };

        assert!(models.contains(&expected_model));
    }
}