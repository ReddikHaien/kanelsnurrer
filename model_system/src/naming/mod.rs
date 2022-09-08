#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct MaterialIdentifier(Box<[Element]>);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Element{
    Custom(String)
}

impl MaterialIdentifier{
    pub fn clone_new_from(vec: &Vec<Element>) -> Self{
        let clone = vec.clone();
        let clone = clone.into_boxed_slice();
        Self(clone)
    }
}