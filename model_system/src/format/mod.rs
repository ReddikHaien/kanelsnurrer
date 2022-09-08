use serde::Deserialize;

#[derive(Deserialize, PartialEq)]
pub struct ModelFile<'a>{
    pub inherits: Option<&'a str>,
    #[serde(borrow)]
    pub shape_kinds: ShapeKinds<'a>
}

#[derive(Deserialize, PartialEq)]
pub struct ShapeKinds<'a>{
    #[serde(borrow)]
    pub wall: Option<ShapeEntry<'a>>,
    #[serde(borrow)]
    pub floor: Option<ShapeEntry<'a>>
}

#[derive(Deserialize, PartialEq)]
pub struct ShapeEntry<'a>{
    pub visibility: VisibilityDefinition,
    #[serde(borrow)]
    pub model: Vec<ModelEntry<'a>>
}

#[derive(Deserialize, PartialEq, Default)]
pub struct VisibilityDefinition{
    pub all: Option<Visibility>,
    pub sides: Option<Visibility>,
    /// y+
    /// 
    pub top: Option<Visibility>,
    /// y-
    /// 
    pub bottom: Option<Visibility>, 
    /// z+
    /// 
    pub front: Option<Visibility>,
    /// z-
    /// 
    pub back: Option<Visibility>,
    /// x+
    /// 
    pub left: Option<Visibility>,
    /// x+
    /// 
    pub right: Option<Visibility>,
}

#[derive(Deserialize, PartialEq)]
pub enum Visibility{
    Solid,
    Transparent
}

#[derive(Deserialize, PartialEq)]
pub struct ModelEntry<'a>{
    pub bound: Bound,

    #[serde(borrow)]
    pub definition: ModelDefinition<'a>,

    #[serde(borrow)]
    pub coloring: Coloring<'a>
}

#[derive(Deserialize, PartialEq)]
pub struct Bound{
    pub min: [f32;3],
    pub max: [f32;3],
}

#[derive(Deserialize, PartialEq)]
pub enum ModelDefinition<'a>{
    SDF(&'a str),
    Solid
}

#[derive(Deserialize, PartialEq)]
pub enum Coloring<'a>{
    UvMapped{
        texture: &'a str
    }
}
