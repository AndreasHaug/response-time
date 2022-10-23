use crate::{
    graph::{Node, RoadLink},
    mongodb_queries,
};
use futures::stream::StreamExt;
use mongodb::{
    bson::{self, doc, Bson},
    options::{ClientOptions, ServerAddress},
    Client, 
};
use std::{
    collections::HashMap,
    error::Error,
};


static DB_NAME: &str = "roaddata";
static NODES: &str = "nodes";
static LINKS: &str = "links";


pub async fn get_links() -> Result<HashMap<String, RoadLink>, Box<dyn Error>> {
    let client_options = ClientOptions::builder()
        .hosts(vec![ServerAddress::parse("localhost:27017").unwrap()])
        .build();
    let client: Client = Client::with_options(client_options)?;
    let db = client.database(DB_NAME);
    let link_collection = db.collection::<RoadLink>(LINKS);

    let mut links: HashMap<String, RoadLink> = HashMap::new();
    let mut cursor = link_collection
        .aggregate(mongodb_queries::get_links(), None)
        .await?;
    while let Some(l) = cursor.next().await {
	let linkdata = l.unwrap();
        let link = bson::from_bson::<RoadLink>(Bson::Document(linkdata)).unwrap();
        links.insert(link.reference.to_owned(), link);
    }

    Ok(links)
}


pub async fn get_nodes() -> Result<HashMap<String, Node>, Box<dyn Error>> {
    let client_options = ClientOptions::builder()
        .hosts(vec![ServerAddress::parse("localhost:27017").unwrap()])
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
