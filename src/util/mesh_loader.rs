
use std::{path::{PathBuf, Path}, borrow::Cow, f32::consts::E, fmt::{format, Debug, Display}, fs::File, error::Error, io::{Read, BufReader, BufRead}};

use bevy::prelude::{Vec3, Vec2};

pub fn load_mesh_file<'a>(path: &str) -> Result<(Vec<Vec3>, Vec<Vec2>, Vec<Vec3>, Vec<u16>), Box<dyn Error>>{
    let path_buf = PathBuf::from(path);

    match path_buf.extension(){
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "obj" => load_wavefront_file(&path_buf),
            x => MeshLoadingError::new(path, format!("Invalid extension: {}",x)).into_boxed_err()
        },
        None => MeshLoadingError::new(path, "No extension found on path").into_boxed_err()
    }
}

fn load_wavefront_file(path: &Path) -> Result<(Vec<Vec3>, Vec<Vec2>, Vec<Vec3>, Vec<u16>), Box<dyn Error>>{
    let file = File::open(path)?;
    let reader = BufReader::new(file).lines();
    
    let mut raw_verts = Vec::new();
    let mut raw_uvs = Vec::new();
    let mut raw_normals = Vec::new();

    let mut verts = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    fn get_index(p: &str, raw_verts: &Vec<Vec3>, raw_uvs: &Vec<Vec2>, raw_normals: &Vec<Vec3>, verts: &mut Vec<Vec3>, uvs: &mut Vec<Vec2>, normals: &mut Vec<Vec3>) -> Result<u16,Box<dyn Error>>{
        
        let individuals = p.split("/").map(|x| x.parse()).try_collect::<Vec<u16>>()?;

        let vert = raw_verts[individuals[0] as usize - 1];
        let uv = raw_uvs[individuals[1] as usize - 1];
        let normal = raw_normals[individuals[2] as usize - 1];

        for i in 0..verts.len(){
            if verts[i] == vert && uvs[i] == uv && normals[i] == normal{
                return Ok(i as u16);
            }
        }

        verts.push(vert);
        uvs.push(uv);
        normals.push(normal);

        Ok(verts.len() as u16 - 1)
    }

    for line in reader{
        let line = line?;
        if line.is_empty() || line.starts_with("#"){
            continue;
        }

        let tokens = line.split(' ').collect::<Vec<_>>();
        let type_ = tokens[0];

        match type_ {
            "v" => {
                let mut x = tokens[1].parse()?;
                let mut y = tokens[2].parse()?;
                let mut z = tokens[3].parse()?;
                if let Some(w) = tokens.get(4){
                    let w: f32 = w.parse()?;
                    x /= w;
                    y /= w;
                    z /= w;
                }

                raw_verts.push(Vec3::new(x, y, z));
            },

            "vt" => {
                let u = tokens[1].parse()?;
                let v = tokens.get(2).map(|x| x.parse()).unwrap_or(Ok(0.0f32))?;
                raw_uvs.push(Vec2::new(u, v));
            }

            "vn" => {
                let x = tokens[1].parse()?;
                let y = tokens[2].parse()?;
                let z = tokens[3].parse()?;
                raw_normals.push(Vec3::new(x, y, z));
            },

            "f" => {
                let a = get_index(tokens[1], &raw_verts, &raw_uvs, &raw_normals, &mut verts, &mut uvs, &mut normals)?;
                let b = get_index(tokens[2], &raw_verts, &raw_uvs, &raw_normals, &mut verts, &mut uvs, &mut normals)?;
                let c = get_index(tokens[3], &raw_verts, &raw_uvs, &raw_normals, &mut verts, &mut uvs, &mut normals)?;
                
                if tokens.len() == 5{
                    let d = get_index(tokens[4], &raw_verts, &raw_uvs, &raw_normals, &mut verts, &mut uvs, &mut normals)?;
                    indices.push(a);
                    indices.push(b);
                    indices.push(c);
                    
                    indices.push(b);
                    indices.push(d);
                    indices.push(a);
                }
                else{
                    indices.push(a);
                    indices.push(b);
                    indices.push(c);
                }
            },
            x => println!("unhandled obj type {}",x)
        };

    }


    Ok((verts,uvs,normals,indices))
}


pub struct MeshLoadingError{
    path: String,
    error: String,
}

impl MeshLoadingError{
    pub fn new(path: impl ToString, error: impl ToString) -> Self{
        Self{
            error: error.to_string(),
            path: path.to_string()
        }
    }

    pub fn into_boxed_err<T>(self) -> Result<T,Box<dyn Error>>{
        Err(Box::new(self))
    }
}

impl Debug for MeshLoadingError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        f.write_str("Error while loading mesh at (")?;
        std::fmt::Debug::fmt(&self.path, f)?;
        f.write_str("): ")?;
        std::fmt::Debug::fmt(&self.error, f)?;
        Ok(())
    }
}

impl Display for MeshLoadingError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Error while loading mesh at (")?;
        std::fmt::Display::fmt(&self.path, f)?;
        f.write_str("): ")?;
        std::fmt::Display::fmt(&self.error, f)?;
        Ok(())
    }
}

impl Error for MeshLoadingError{}