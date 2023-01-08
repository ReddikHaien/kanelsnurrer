use std::{collections::BTreeMap, fmt::Debug, borrow::Borrow};

use crate::util::result_ext::ResultExt;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum MaterialIdentifierElement {
    Custom(String),
}

impl MaterialIdentifierElement{
    pub fn as_str(&self) -> &str{
        match self{
            MaterialIdentifierElement::Custom(s) => s,
        }
    }

    pub fn is_ignorable(&self) -> bool{
        match self {
            MaterialIdentifierElement::Custom(s) => s == "STRUCTURAL",
        }
    }
}

impl<T: Into<String>> From<T> for MaterialIdentifierElement {
    fn from(s: T) -> Self {
        let mut s: String = s.into();
        s.make_ascii_uppercase();
        Self::Custom(s)
    }
}

pub struct MaterialIdentifier(Box<[MaterialIdentifierElement]>);

impl From<String> for MaterialIdentifier {
    fn from(s: String) -> Self {
        let v = s
            .split(':')
            .map(|x| x.into())
            .collect::<Vec<MaterialIdentifierElement>>();
        Self(v.into_boxed_slice())
    }
}

impl From<Vec<MaterialIdentifierElement>> for MaterialIdentifier {
    fn from(v: Vec<MaterialIdentifierElement>) -> Self {
        Self(v.into_boxed_slice())
    }
}

impl Debug for MaterialIdentifier{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dt = f.debug_tuple("");
        for x in self.0.iter(){
            dt.field(x);
        }

        dt.finish()
    }
}

pub struct MaterialIdentifierStorage {
    storage: StoragEntry,
}

impl MaterialIdentifierStorage {
    pub fn new() -> Self {
        Self {
            storage: StoragEntry::Branch {
                children: BTreeMap::new(),
                default_model: 0,
            },
        }
    }

    pub fn get_id(&self, identifier: &MaterialIdentifier) -> Result<u32,u32> {
        self.storage.get(&identifier.0)
    }

    pub fn set_id(&mut self, identifier: &MaterialIdentifier, value: u32) {
        self.storage.set(&identifier.0, value);
    }

    pub fn print_tree(&self) {
        self.storage.print(0);
    }
}

enum StoragEntry {
    Leaf(u32),
    Branch {
        children: BTreeMap<MaterialIdentifierElement, StoragEntry>,
        default_model: u32,
    },
}

impl StoragEntry {
    fn print(&self, indentation: usize) {
        match self {
            StoragEntry::Leaf(model) => {
                println!(": {}", model);
            }
            StoragEntry::Branch {
                children,
                default_model,
            } => {
                println!(": {}", default_model);
                for x in children {
                    for _ in 0..indentation {
                        print!(" ");
                    }
                    print!("{:?}", x.0);
                    x.1.print(indentation + 4);
                }
            }
        }
    }

    fn get(&self, identifier: &[MaterialIdentifierElement]) -> Result<u32,u32> {
        match self {
            StoragEntry::Leaf(id) => {
                if identifier.len() > 0 && !identifier[0].is_ignorable(){
                    Err(*id)
                }
                else{
                    Ok(*id)
                }
            },
            StoragEntry::Branch {
                children,
                default_model,
            } => {
                if identifier.len() == 0 {
                    Ok(*default_model)
                } else {
                    match children.get(&identifier[0]) {
                        Some(entry) => match entry.get(&identifier[1..]){
                            x if x.clone().either() == 0 => {
                                Err(*default_model)
                            }
                            x => x
                        },
                        None =>{
                            if identifier[0].is_ignorable(){
                                Ok(*default_model)
                            }
                            else{
                                Err(*default_model)
                            }
                        }
                    }
                }
            }
        }
    }

    fn set(&mut self, identifier: &[MaterialIdentifierElement], value: u32) {

        println!("remaining: {:?}", identifier);
        match self {
            StoragEntry::Leaf(id) => {
                if identifier.len() == 0 {
                    *id = value;
                } else {
                    let mut children = BTreeMap::new();

                    children.insert(
                        identifier[0].clone(),
                        Self::determine_node_type(identifier.len()),
                    );

                    *self = StoragEntry::Branch {
                        children,
                        default_model: *id,
                    }
                }
            }
            StoragEntry::Branch {
                children,
                default_model,
            } => {
                //If this is our last stop, assign the default model
                if identifier.len() == 0 {
                    *default_model = value
                } else {
                    match children.get_mut(&identifier[0]) {
                        Some(x) => x.set(&identifier[1..], value),
                        None => {
                            children.insert(
                                identifier[0].clone(),
                                Self::determine_node_type(identifier.len()),
                            );
                            children.get_mut(&identifier[0]).unwrap().set(
                                &identifier[1..],
                                value,
                            );
                        }
                    }
                }
            }
        }
    }

    fn determine_node_type(left: usize) -> StoragEntry {
        if left >= 2 {
            StoragEntry::Branch {
                children: BTreeMap::new(),
                default_model: 0,
            }
        } else {
            StoragEntry::Leaf(0)
        }
    }
}
