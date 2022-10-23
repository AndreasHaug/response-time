use std::fmt::Debug;
use serde::Serialize;
use crate::{graph::NodeCost};

#[derive(Serialize, Debug)]
pub enum SearchResultProperty {
    MultilinestringResult(MultiLineStringResult),
    PolygonResult(PolygonResult),
    DebugResult(DebugResult),
}

#[derive(Serialize, Debug)]
pub struct Start {
    pub r#type : String,
    pub coordinates: [f64; 2],
}

#[derive(Serialize, Debug)]
pub struct SearchResult {
    start: Start,
    result: SearchResultProperty,
}

#[derive(Serialize, Debug)]
pub struct MultiLineStringResult {
    // start: Start,
    r#type: String,
    coordinates: Vec<Vec<[f64; 2]>>,
}

#[derive(Serialize, Debug)]
pub struct PolygonResult {
    // start: [f64; 2],
    r#type: String,
    coordinates: Vec<[f64; 2]>,
}

pub struct MultiPointResult {
    r#type: String,
    coordinates: Vec<[f64; 2]>,
}

#[derive(Serialize, Debug)]
pub struct DebugResult {
    nodes: Vec<NodeCost>,
    polygon: PolygonResult,
    multilinestring: MultiLineStringResult,
}

impl SearchResult {
    pub fn new(start: Start, result: SearchResultProperty) -> SearchResult {
	SearchResult {
	    start,
	    result,
	}
    }
}


impl DebugResult {
    pub fn new(nodes: Vec<NodeCost>, polygon: PolygonResult, multilinestring: MultiLineStringResult) -> Self {
	DebugResult { nodes, polygon, multilinestring }
    }
}

impl MultiLineStringResult {
    pub fn new(coordinates: Vec<Vec<[f64; 2]>>) -> Self {
	MultiLineStringResult {
	    r#type: String::from("MultiLineString"),
	    coordinates,
	}
    }
}

impl PolygonResult {
    pub fn new(coordinates: Vec<[f64; 2]>) -> Self {
	PolygonResult {
	    r#type: String::from("Polygon"),
	    coordinates,
	}
    }
}

impl MultiPointResult {
    pub fn new(coordinates: Vec<[f64; 2]>) -> Self {
	MultiPointResult {
	    // start,
	    r#type: String::from("MultiPoint"),
	    coordinates,
	}
    }
}
