#![feature(decl_macro, proc_macro_hygiene)]

use std::{
    path::{Path},
    time::Instant,
};


use std::env;

use main::{search_result::*, db};

use mongodb::Client;
use mongodb::options::{ClientOptions, ServerAddress};

use main::{
    closest::Closest,
    graph::{Graph},
};
use rocket::fs::NamedFile;
use rocket::State;


use rocket::form::FromForm;
use rocket::{get, post, serde::json::Json};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi, openapi_get_routes, rapidoc::*, swagger_ui::*};
use serde::{Deserialize, Serialize};


#[macro_use] extern crate rocket;

#[openapi(skip)]
#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/static/index.html"))
        .await
        .ok()
}


#[openapi(skip)]
#[get("/styles")]
async fn styles() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/css/styles.css")).await.ok()
}


#[openapi(skip)]
#[get("/normalize")]
async fn normalize() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/css/normalize.css")).await.ok()
}


#[openapi(skip)]
#[get("/map")]
async fn map() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/js/map.js")).await.ok()
}


#[openapi(skip)]
#[get("/navigationbar")]
async fn navigationbar() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/js/navigationbar.js")).await.ok()
}


#[openapi(tag = "Polygon")]
#[get("/polygon?<lat>&<lng>&<cost>")]
async fn polygon(graph: &State<Graph>,
		 lat: f64,
		 lng: f64,
		 cost: i32) -> Option<Json<SearchResult>> {
    
    let s = Instant::now();
    println!("max cost: {}", cost);
    let res = Some(Json(graph
			.search(closest(lat, lng).await.unwrap(), cost)
			.await
			.unwrap()
			.as_polygon()));
    println!("{:?}", s.elapsed());
    res
}


async fn closest(lat: f64, lng: f64) -> Result<Closest, reqwest::Error> {
    reqwest::get(format!("{}?lat={}&lng={}", env::var("CLOSEST_URL").unwrap(), lat, lng))
	.await?
	.json::<Closest>()
	.await
}


#[openapi(tag = "MultiLineString")]
#[get("/multilinestring?<lat>&<lng>&<cost>")]
async fn multilinestring(graph: &State<Graph>,
			 lat: f64,
			 lng: f64,
			 cost: i32) -> Option<Json<SearchResult>> {
    
    
    let s = Instant::now();
    let res = Some(Json(
        graph
            .search(closest(lat, lng).await.unwrap(), cost)
            .await
            .unwrap()
            .as_multilinestring()));
    println!("{:?}", s.elapsed());
    res
}


#[launch]
async fn launch() -> _ {

    let connection_string = env::var("MONGO_DB_CONNECTION").unwrap();
    let client_options = ClientOptions::builder()
	.hosts(vec![ServerAddress::parse(connection_string).unwrap()])
	// .credential(mongodb::options::Credential::builder()
	// 	    .username(Some(env::var("MONGO_USERNAME").unwrap()))
	// 	    .source(Some("roaddata".to_string()))
	// 	    .password(Some(env::var("MONGO_PASSWORD").unwrap()))
	// 	    .build()
	// )
        .build();

    println!("Loading data.");
    println!("  Loading nodes.");
    let nodes = db::get_nodes().await.unwrap();
    println!("  Finished loading nodes");
    println!("  Loading links.");
    let links = db::get_links().await.unwrap();
    println!("  Finished loading links");
    println!("Starting server.");

    

    let client: Client = Client::with_options(client_options).unwrap();
    
    rocket::build()
        .manage(Graph::new(nodes, links).await)
        .manage(
	    client
        )
        .mount(
	    "/",
	    openapi_get_routes![
		polygon,
		multilinestring,
		index,
		styles,
		normalize,
		navigationbar,
		map
	    ])
        .mount(
	    "/api/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),	    
	)        
}
