use crate::{
    closest::{self, Closest},
    graph::{self, Graph, Node, RoadLink},
    mongodb_queries,
};
use futures::stream::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{self, doc, to_bson, to_document, Bson, Document},
    options::{ClientOptions, Credential, FindOptions, ServerAddress},
    Client, Collection, Cursor, Database, IndexModel,
};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::read_dir,
    path::Path,
};
// use mongodb::{Client, Collection};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

static DB_NAME: &str = "roaddata";
static NODES: &str = "nodes";
static RAW_LINKS: &str = "raw_links";
static LINKS: &str = "links";
static LINKPATH: &str = "./nvdb_client/DATA/veglenkesekvenser/";

// static CLIENT_RUNTIME: Lazy<(Client, Runtime)> = Lazy::new(|| {
//     let rt = Runtime::new().unwrap();
//     let client = rt.block_on(async {
// 	// Client = Client::with_uri_str("localhost:27017")
// 	Client::with_uri_str("mongodb://127.0.0.1:27017/").await.unwrap()
//     });
//     (client, rt)
// });

pub async fn read_raw_links(path: &str) -> Result<(), Box<dyn Error>> {
    let mut client_options = ClientOptions::builder()
        // .credential(Credential::builder().username("skrivebruker").password("skrivepassord").)
        .hosts(vec![ServerAddress::parse("localhost:27017").unwrap()])
        .build();
    let client: Client = Client::with_options(client_options)?;
    let db = client.database(DB_NAME);
    let collection = db.collection::<Value>(RAW_LINKS);
    collection.drop(None).await?;
    for entry in read_dir(LINKPATH).unwrap() {
        let p = entry.unwrap().path();
        println!("Adding {:?}", p);
        let val: Value = match serde_json::from_str(
            &std::fs::read_to_string(&p).expect("could not read file"),
        ) {
            Ok(o) => o,
            Err(e) => panic!("{:?}", p),
        };
        let vals: &Vec<Value> = val["objekter"].as_array().unwrap();
        match collection.insert_many(vals, None).await {
            Ok(o) => {}
            Err(_) => {
                if vals.len() == 0 {
                    continue;
                } else {
                    panic!("{:?}", vals)
                }
            }
        };
    }
    Ok(())
}

// new Map([["type", "LineString"], ["coordinates", s.slice(13, -1).split(", ").map(r => r.split(" ")).map(p => [p[0], p[1]])]])
pub async fn get_links() -> Result<HashMap<String, RoadLink>, Box<dyn Error>> {
    let mut client_options = ClientOptions::builder()
        .hosts(vec![ServerAddress::parse("localhost:27017").unwrap()])
        .build();
    let client: Client = Client::with_options(client_options)?;
    let db = client.database(DB_NAME);
    let link_collection = db.collection::<RoadLink>(LINKS);

    let mut links: HashMap<String, RoadLink> = HashMap::new();
    // let mut cursor = link_collection.find(doc! {}, None).await?;
    let mut cursor = link_collection
        .aggregate(mongodb_queries::get_links(), None)
        .await?;
    while let Some(l) = cursor.next().await {
        // let link = l.unwrap();

	let linkdata = l.unwrap();
	// println!("{:?}", linkdata);
        let link = bson::from_bson::<RoadLink>(Bson::Document(linkdata)).unwrap();
        links.insert(link.reference.to_owned(), link);
    }

    Ok(links)
}

pub async fn closest_link(lat: f64, lon: f64) -> Result<Option<Closest>, Box<dyn Error>> {
    // let (client, rt) = &*CLIENT_RUNTIME;
    let mut client_options = ClientOptions::builder()
        .hosts(vec![ServerAddress::parse("localhost:27017").unwrap()])
        .build();
    let client: Client = Client::with_options(client_options)?;
    let database = client.database(DB_NAME);
    let collection = database.collection::<Document>(LINKS);

    // rt.block_on(async {
    let mut cursor = collection
        .aggregate(mongodb_queries::closest_link(lat, lon), None)
        .await;
    if let Ok(mut o) = cursor {
        let next = o.next().await.ok_or("")??;
        // println!("{:?}", next);
        let closest = bson::from_bson::<Closest>(Bson::Document(next)).unwrap();
        return Ok(Some(closest));
    }
    return Ok(None);

    // }
    // })
    // bson::from_bson::<RoadLink>(Bson::Document(l.unwrap())).unwrap();

    // let c = mongodb_queries::closest_link(lat, lon);
    // None
}

pub async fn get_nodes() -> Result<HashMap<String, Node>, Box<dyn Error>> {
    let mut client_options = ClientOptions::builder()
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

pub async fn load_nodes() -> Result<(), Box<dyn Error>> {
    //finne alle node-id'er
    let mut client_options = ClientOptions::builder()
        // .credential(Credential::builder().username("skrivebruker").password("skrivepassord").)
        .hosts(vec![ServerAddress::parse("localhost:27017").unwrap()])
        .build();

    let client: Client = Client::with_options(client_options)?;
    let db = client.database(DB_NAME);
    let link_collection = db.collection::<RoadLink>(LINKS);
    let node_collection = db.collection::<Node>(NODES);
    node_collection.drop(None).await?;

    let mut node_ids: HashSet<String> = HashSet::new();
    let mut cursor = link_collection.find(doc! {}, None).await?;
    // let mut cursor = link_collection.aggregate(vec![
    // 	doc!{
    // 	    "$project" :
    // 	    doc! {
    // 		"startnode" : "$startnode",
    // 		"endnode" : "$endnode"
    // 	    },
    // 	}
    // ], None
    // ).await?;

    while let Some(l) = cursor.next().await {
        // println!("{:?}", l);
        // let link = bson::from_bson::<RoadLink>(Bson::Document(l.unwrap())).unwrap();
        let link = l.unwrap();
        // let mut link =
        node_ids.insert(link.startnode().to_owned());
        node_ids.insert(link.endnode().to_owned());
    }

    // let mut nodes: HashMap<String, Node> = HashMap::new();
    println!("Read all node ids");

    for a in node_ids.into_iter() {
        let filter = doc! { "$or" : vec! [ doc!{ "startnode" : &a }, doc! { "endnode" : &a } ] };
        let find_options = FindOptions::builder().sort(doc! { "reference": 1 }).build();
        let mut cursor = link_collection.find(filter, find_options).await?;

        let mut link_refs: Vec<String> = Vec::new();
        while let Some(l) = cursor.next().await {
            let link: RoadLink = l.unwrap();
            // println!("{:?}", link);
            link_refs.push(link.reference);
        }
        // println!("{:?}", link_refs);
        // nodes.insert(a.to_owned(), Node::new(a, link_refs));

        node_collection
            .insert_one(Node::new(a, link_refs), None)
            .await
            .unwrap();
    }

    Ok(())
}

// pub async fn load_links() -> Result<(), Box<dyn Error>> {
//     let mut client_options = ClientOptions::builder()
//         // .credential(Credential::builder().username("skrivebruker").password("skrivepassord").)
//         .hosts(vec![ServerAddress::parse("localhost:27017").unwrap()])
//         .build();
//     let client: Client = Client::with_options(client_options)?;
//     let db = client.database(DB_NAME);
//     let from_collection = db.collection::<Document>(RAW_LINKS);

//     let to_collection = db.collection::<RoadLink>(LINKS);
//     to_collection.drop(None).await?;

//     let mut cursor = from_collection
//         .aggregate(mongodb_queries::extract_from_rawlink(), None)
//         .await?;
//     while let Some(l) = cursor.next().await {
//         let mut link = bson::from_bson::<RoadLink>(Bson::Document(l.unwrap())).unwrap();

// 	// if link.reference != "858886-2-5" && link.reference != "858886-3-5" {
// 	// if !link.reference.starts_with("858886") && !link.reference.starts_with("858888") {
	
//         link.utm33_to_lat_lon();
	
	
//         to_collection.insert_one(link, None).await?;
// 	// }
//     }
//     to_collection
//         .create_index(
//             IndexModel::builder().keys(doc! {"reference" : 1}).build(),
//             None,
//         )
//         .await?;
//     to_collection
//         .create_index(
//             IndexModel::builder().keys(doc! {"startnode" : 1}).build(),
//             None,
//         )
//         .await?;
//     to_collection
//         .create_index(
//             IndexModel::builder().keys(doc! {"endnode" : 1}).build(),
//             None,
//         )
//         .await?;
//     to_collection
//         .create_index(
//             IndexModel::builder()
//                 .keys(doc! {"geometry" : "2dsphere"})
//                 .build(),
//             None,
//         )
//         .await?;

//     Ok(())
// }

pub mod tests {

    use geomorph::utm;
    #[test]
    pub fn test_utm33_latlon_conversion() {
        let u: utm::Utm = utm::Utm::new(155306.26, 6527946.062, true, 33, 'N', false);
        let c: geomorph::coord::Coord = u.clone().into();
        // println!("{}", c);
        // println!("{}", u);

        // let c = geomorph::coord::Coord::new(59.9320277, 10.8986159);
        // println!("{}", c);
        // let u: utm::Utm = c.clone().into();
        // 155306.26 6527946.062
        // println!("testing lololol");
        // println!("{}", u);
    }

    // pub fn get_closest_node() {

    // }
}
