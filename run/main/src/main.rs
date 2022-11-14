#![feature(decl_macro, proc_macro_hygiene)]

use std::{
    path::{Path},
    time::Instant,
};


use std::env;

use main::{search_result::*, db};

use mongodb::Client;
use main::{
    closest::Closest,
    graph::{Graph},
};
use rocket::fs::NamedFile;
use rocket::serde::json::{Json};
use rocket::State;
use rocket::get;
// use rocket_contrib::json::Json;
// use rocket_contrib::json::Json as RocketJson;
// use rocket_okapi::{openapi, routes_with_openapi, JsonSchema};
// use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
// use rocket_okapi::{swagger_ui::*, openapi};
// extern crate dotenv;
// use rocket_okapi::{openapi, openapi_get_routes};

#[macro_use] extern crate rocket;

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/static/index.html"))
        .await
        .ok()
}


#[get("/")]
async fn styles() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/css/styles.css")).await.ok()
}


#[get("/")]
async fn normalize() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/css/normalize.css")).await.ok()
}


#[get("/")]
async fn map() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/js/map.js")).await.ok()
}


#[get("/")]
async fn navigationbar() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/js/navigationbar.js")).await.ok()
}

// #[openapi]
#[get("/?<lat>&<lng>&<cost>")]
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

// "http://closest:8001/?lat={}&lng={}"
async fn closest(lat: f64, lng: f64) -> Result<Closest, reqwest::Error> {
    println!("{}", env::var("CLOSEST_URL").unwrap());
    reqwest::get(format!("{}?lat={}&lng={}", env::var("CLOSEST_URL").unwrap(), lat, lng))
    // reqwest::get(format!("http://localhost:8001/?lat={}&lng={}", lat, lng))
	// reqwest::get(format!("http://closest:8001/?lat={}&lng={}?lat={}&lng={}", lat, lng))
	.await?
	.json::<Closest>()
	.await
}


#[get("/?<lat>&<lng>&<cost>")]
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
	.credential(mongodb::options::Credential::builder()
		    .username(Some(env::var("MONGO_USERNAME").unwrap()))
		    .source(Some("roaddata".to_string()))
		    .password(Some(env::var("MONGO_PASSWORD").unwrap()))
		    .build()
	)
        .build();

    println!("Loading data.");
    println!("  Loading nodes.");
    let nodes = db::get_nodes().await.unwrap();
    println!("  Finished loading nodes");
    println!("  Loading links.");
    let links = db::get_links().await.unwrap();
    println!("  Finished loading links");
    println!("Starting server.");
    
    rocket::build()
        .manage(Graph::new(nodes, links).await)
        .manage(
            Client::with_uri_str("mongodb://127.0.0.1:27017/")
                .await
                .unwrap(),
        )
        .mount("/", routes![index])
	.mount("/styles", routes![styles])
        .mount("/normalize", routes![normalize])
	.mount("/navigationbar", routes![navigationbar])
        .mount("/map", routes![map])
        .mount("/polygon", routes![polygon])
        .mount("/multilinestring", routes![multilinestring])

}
