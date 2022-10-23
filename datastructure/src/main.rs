use std::{
    path::{Path},
    time::Instant,
};

use datastructure::{search_result::*, db};

use mongodb::Client;
use datastructure::{
    closest::Closest,
    graph::{Graph},
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

async fn closest(lat: f64, lng: f64) -> Result<Closest, reqwest::Error> {
    reqwest::get(format!("http://localhost:8001/?lat={}&lng={}", lat, lng))
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

    let nodes = db::get_nodes().await.unwrap();
    let links = db::get_links().await.unwrap();
    
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
