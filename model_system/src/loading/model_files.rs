use std::collections::BTreeMap;

use crate::{format::ModelFile, naming::MaterialIdentifier};

pub struct LoadedModels<'a>{
    entries: BTreeMap<&'a MaterialIdentifier, ModelFile<'a>>
}

impl<'a> LoadedModels<'a>{
    pub fn new(entries: BTreeMap<&'a MaterialIdentifier, ModelFile<'a>>) -> Self{
        Self {
            entries
        }
    }
}

#[cfg(test)]
impl<'a> LoadedModels<'a>{
    pub fn num_entries(&self) -> usize{
        self.entries.len()
    }

    pub fn contains<'b>(&self, file: &ModelFile<'b>) -> bool{
        for x in self.entries.values(){
            if x.eq(file){
                return true;
            }
        }
        false
    }
}