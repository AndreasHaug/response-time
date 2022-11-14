use std::fmt::Debug;
// use schemars::JsonSchema;
// use rocket_okapi::JsonSchema;

use rocket::form::FromForm;
use rocket::{get, post, serde::json::Json};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi, openapi_get_routes, rapidoc::*, swagger_ui::*};
use serde::{Deserialize, Serialize};



// use serde::Serialize;

#[derive(Serialize, Debug, JsonSchema)]
pub enum SearchResultProperty {
    MultilinestringResult(MultiLineStringResult),
    PolygonResult(PolygonResult),
}


#[derive(Serialize, Debug, JsonSchema)]
pub struct Start {
    pub r#type : String,
    pub coordinates: [f64; 2],
}


#[derive(Serialize, Debug, JsonSchema)]
pub struct SearchResult {
    start: Start,
    result: SearchResultProperty,
}


#[derive(Serialize, Debug, JsonSchema)]
pub struct MultiLineStringResult {
    // start: Start,
    r#type: String,
    coordinates: Vec<Vec<[f64; 2]>>,
}


#[derive(Serialize, Debug, JsonSchema)]
pub struct PolygonResult {
    // start: [f64; 2],
    r#type: String,
    coordinates: Vec<Vec<[f64; 2]>>,
}


impl SearchResult {
    pub fn new(start: Start, result: SearchResultProperty) -> SearchResult {
	SearchResult {
	    start,
	    result,
	}
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
	    coordinates: vec![coordinates],
	}
    }
}
