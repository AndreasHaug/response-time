use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    r#type: String,
    pub coordinates: [f64; 2],
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Closest {
    pub link: String,
    pub geometry: Point,
    pub node: Option<String>,
    pub linestring_index: usize,
    pub linestring_length: usize,
}


impl Closest {
    pub fn link(&self) -> &String {
        &self.link
    }

    
    pub fn linestring_index(&self) -> usize {
        self.linestring_index
    }

    
    pub fn linestring_length(&self) -> usize {
        self.linestring_length
    }

    
    pub fn node(&self) -> Option<String> {
	match &self.node {
	    Some(s) => Some(s.to_owned()),
	    None => None,
	}
    }
}
