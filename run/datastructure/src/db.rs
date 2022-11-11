use std::vec;

// use mongodb::bson::{doc, Document};
use crate::{
    graph::{Node, RoadLink},
    // mongodb_queries,
};
use futures::stream::StreamExt;
use mongodb::{
    bson::{self, doc, Bson, Document},
    options::{ClientOptions, ServerAddress},
    Client, 
};
use std::{
    collections::HashMap,
    error::Error,
};

use std::env;


static DB_NAME: &str = "roaddata";
static NODES: &str = "nodes";
static LINKS: &str = "links";




pub(crate) fn get_links_query() -> Vec<Document> {
    vec![doc! {
        "$project" : {
            "reference" : "$reference",
	    "seq_id" : "$seq_id",
            "startnode" : "$startnode",
            "endnode" : "$endnode",
	    "startposition" : "$startposition",
	    "endposition" : "$endposition",
            "length" : "$length",
            "lanes" : "$lanes",
            "geometry" : {
		"type" : "$geometry.type",
		"coordinates" : {
		    "$map" : {
			"input" : "$geometry.coordinates",
			"as" : "coords",
			"in" : { "$reverseArray" : "$$coords"}
		    }
		}
            },
	    "speedlimits" : "$speedlimits"
        }
    }]
}




pub async fn get_links() -> Result<HashMap<String, RoadLink>, Box<dyn Error>> {
    let connection_string = env::var("MONGO_DB_CONNECTION").unwrap();
    let client_options = ClientOptions::builder()
	// .hosts(vec![ServerAddress::parse(env::var("MONGO_DB_CONNECTION").unwrap()).unwrap()])	
    // .hosts(vec![ServerAddress::parse("localhost:27017").unwrap()])	
    // .hosts(vec![ServerAddress::parse("database:27017").unwrap()])
	.hosts(vec![ServerAddress::parse(connection_string).unwrap()])	
        .build();
    let client: Client = Client::with_options(client_options)?;
    let db = client.database(DB_NAME);
    let link_collection = db.collection::<RoadLink>(LINKS);

    let mut links: HashMap<String, RoadLink> = HashMap::new();
    let mut cursor = link_collection
        .aggregate(get_links_query(), None)
        .await?;
    while let Some(l) = cursor.next().await {
	let linkdata = l.unwrap();
        let link = bson::from_bson::<RoadLink>(Bson::Document(linkdata)).unwrap();
        links.insert(link.reference.to_owned(), link);
    }

    Ok(links)
}


pub async fn get_nodes() -> Result<HashMap<String, Node>, Box<dyn Error>> {
    let connection_string = env::var("MONGO_DB_CONNECTION").unwrap();
    
    println!("{}", connection_string);
    let client_options = ClientOptions::builder()
	.hosts(vec![ServerAddress::parse(connection_string).unwrap()])	
    // .hosts(vec![ServerAddress::parse("localhost:27017").unwrap()])
	// .hosts(vec![ServerAddress::parse("database:27017").unwrap()])
        .build();
    let client: Client = Client::with_options(client_options)?;
    let db = client.database(DB_NAME);
    let node_collection = db.collection::<Node>(NODES);

    let mut nodes: HashMap<String, Node> = HashMap::new();
    let mut cursor = node_collection.find(doc! {}, None).await?;

    while let Some(n) = cursor.next().await {
        let node: Node = n.unwrap();
        nodes.insert(node.id.to_owned(), node);
    }
    Ok(nodes)
}
