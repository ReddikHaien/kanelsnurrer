use std::{collections::BTreeMap, borrow::Borrow, sync::Arc, fmt::{Debug, Display}};


use crate::util::{result_ext::ResultExt, cache::CacheKey, display_iter::{Displayable, DisplayableExt}};

#[derive(Debug,Clone)]
pub struct InnerIdentifier(Arc<[MaterialIdentifierElement]>,u32);

impl Display for InnerIdentifier{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let displayable = self.0[0..self.1 as usize].iter().into_displayable("::");
        Display::fmt(&displayable, f)
    }
}

impl PartialEq for InnerIdentifier{
    fn eq(&self, other: &Self) -> bool {
        if self.1 == other.1{
            for i in 0..self.1{
                let a = &self.0[i as usize];
                let b = &other.0[i as usize];
                if a != b{
                    return false;
                }
            }

            true
        }
        else{
            false
        }
    }
}

impl PartialOrd for InnerIdentifier{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.1.partial_cmp(&other.1) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        
        for i in 0..self.1{
            let a = &self.0[i as usize];
            let b = &other.0[i as usize];
            match a.partial_cmp(b){
                Some(core::cmp::Ordering::Equal) => {},
                ord => return ord,
            }
        }

        Some(core::cmp::Ordering::Equal)
    }
}

impl Eq for InnerIdentifier{}
impl Ord for InnerIdentifier{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Identifier(InnerIdentifier);

impl Debug for Identifier{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Display for Identifier{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl CacheKey for Identifier{
    fn parent(&self) -> Option<Self> {
        let els = (self.0).1 - 1;
        if els == 0{
            None
        }
        else{
            Some(Identifier(InnerIdentifier((self.0).0.clone(),els)))
        }
    }
}

impl Identifier{
    pub fn is_empty(&self) -> bool{
        (self.0).1 == 0
    }

    pub fn last(&self) -> Option<&str>{
        if (self.0).1 == 0{
            None
        }
        else{
            Some((self.0).0[(self.0).1 as usize - 1].as_str())
        }
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        let v = s
            .split(':')
            .map(|x| x.into())
            .collect::<Vec<MaterialIdentifierElement>>();

        let content: Arc<[MaterialIdentifierElement]> = Arc::from(v.into_boxed_slice());
        let c = content.len() as u32;
        Self(InnerIdentifier(content,c))
    }
}

impl From<Vec<MaterialIdentifierElement>> for Identifier{
    fn from(value: Vec<MaterialIdentifierElement>) -> Self {
        let len = value.len() as u32;
        Self(InnerIdentifier(value.into_boxed_slice().into(),len))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum MaterialIdentifierElement {
    Custom(String),
}

impl Display for MaterialIdentifierElement{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            MaterialIdentifierElement::Custom(str) => Display::fmt(str, f),
        }
    }
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
