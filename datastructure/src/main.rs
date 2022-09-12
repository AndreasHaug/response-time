use std::{
    path::{Path},
    time::Instant,
};

use datastructure::search_result::*;

use mongodb::Client;
use datastructure::{
    closest::Closest,
    graph::{Graph},
    mongodb_interaction,
};
use rocket::fs::NamedFile;
use rocket::serde::json::{Json};
use rocket::State;

#[macro_use] extern crate rocket;

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/static/index.html"))
        .await
        .ok()
}

#[get("/")]
async fn map() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend/js/map.js")).await.ok()
}

#[get("/?<lat>&<lon>&<cost>")]
async fn polygon(graph: &State<Graph>,
		 lat: f64,
		 lon: f64,
		 cost: i32) -> Option<Json<SearchResult>> {
    
    let s = Instant::now();

    let res = Some(Json(graph
			.search(closest(lat, lon).await.unwrap(), cost)
			.await
			.unwrap()
			.as_polygon()));
    println!("{:?}", s.elapsed());
    res
}

async fn closest(lat: f64, lng: f64) -> Result<Closest, reqwest::Error> {
    reqwest::get(format!("http://localhost:8001/?lat={}&lng={}", lat, lng))
	.await?
	.json::<Closest>()
	.await
}


#[get("/?<lat>&<lon>&<cost>")]
async fn multilinestring(graph: &State<Graph>,
			 client: &State<Client>,
			 lat: f64,
			 lon: f64,
			 cost: i32) -> Option<Json<SearchResult>> {
    
    let s = Instant::now();
    let res = Some(Json(
        graph
            .search(closest(lat, lon).await.unwrap(), cost)
            .await
            .unwrap()
            .as_multilinestring()));
    println!("{:?}", s.elapsed());
    res
}


#[launch]
async fn launch() -> _ {

    let nodes = mongodb_interaction::get_nodes().await.unwrap();
    let links = mongodb_interaction::get_links().await.unwrap();
    
    rocket::build()
        .manage(Graph::new(nodes, links).await)
        .manage(
            Client::with_uri_str("mongodb://127.0.0.1:27017/")
                .await
                .unwrap(),
        )
        .mount("/", routes![index])
        .mount("/map", routes![map])
        .mount("/polygon", routes![polygon])
        .mount("/multilinestring", routes![multilinestring])
}
