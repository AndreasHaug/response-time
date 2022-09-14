//implement closest
//find closest link
//find list of coordinates
//estimate closest coordinate, get index

//idea:
//if closest is a node, that is, if closest point is either first or
// last of the linestring, we add this node to Dijkstra queue with
// cost 0

//else we find distance on linestring in both directions we
// can drive and add to Dijkstra queue with cost = distance from
// startpoint. Must also check which directions one can drive.

//need:
// - distance of linestring formula
// - distance between points formula

// create a query function in load_mongodb
// takes a collection name
// do the query, return a cursor

//we query with find closest, get a link
//find closest point on the linestring
//find index
//can this be done in a mongodb query?
//use some $function, map to distance from point and then return min
// of these distances

use crate::mongodb_queries;
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
    
    // pub start: [f64; 2],
    // pub reference: String,
    // pub linestring_index: i32,
    // pub linestring_length: i32,
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
	// &self.node
	match &self.node {
	    Some(s) => Some(s.to_owned()),
	    None => None,
	}
    }
}
// finn en referanse
// fn haversine_distance(p1: [f64; 2], p2: [f64; 2]) {}
