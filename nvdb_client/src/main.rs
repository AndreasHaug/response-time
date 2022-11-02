use reqwest::{Client, RequestBuilder, header::HeaderMap, Response, Error};
use serde_json::Value;
use std::{env, path::{Path, PathBuf}, fs::File};
use tokio::runtime::Runtime;

// const FILE_DEST: &str = "./DATA";


#[tokio::main]
async fn main() {

    let LINKSEQ_DEST: &Path = Path::new("../DATA/veglenkesekvenser");
    let ROADLIMIT_DEST: &Path = Path::new("../DATA/fartsgrenser");


    let args: Vec<String> = env::args().collect();
    if !Path::exists(LINKSEQ_DEST) {
	std::fs::create_dir_all(LINKSEQ_DEST);
    }
    if !Path::exists(ROADLIMIT_DEST) {
	std::fs::create_dir_all(ROADLIMIT_DEST);
    }
    
	
    let client: Client = Client::new();

    get_links(&client).await;
    get_speedlimits(&client).await;
    
}

async fn init_request(url: &str, client: &Client, x_client: &str, params: &[(&str, &str)]) -> Result<Response, Error> {
    Ok(client
        .get(url)
        .header("X-Client", x_client)
        .query(params)
        .send()
        .await?)
}

async fn request(client: &Client, url: &str, x_client: &str) -> Result<Response, Error> {
    Ok(client.get(url).header("X-Client", x_client).send().await?)
}

async fn get(client: &Client,  url: &str, x_client: &str, dest: &str, params: &[(&str, &str)]) -> Result<Value, Error> {
    match init_request(url, &client, x_client, params).await {
	Ok(o) => {
	    Ok(o.json::<Value>().await?)
	},
	Err(e) => panic!("{}", e),
    }
}

async fn get_links(client: &Client) {
    requests(client,
	     "https://nvdbapiles-v3.atlas.vegvesen.no/vegnett/veglenkesekvenser/segmentert",
	     Path::new("./DATA/veglenkesekvenser"),
	     "Veglenkeklient",
	     &[("srid", "utm33")]
    ).await;
}

async fn get_speedlimits(client: &Client) {
    requests(client,
	     "https://nvdbapiles-v3.atlas.vegvesen.no/vegobjekter/105/",
	     Path::new("./DATA/fartsgrenser"),
	     "Fartsgrenseklient",
	     &[("inkluder", "alle"), ("srid", "utm33")]
    ).await;
}


// async fn get_speed_limits(client: &Client) {
    // requests(client,
	     // "https://nvdbapiles-v3.atlas.vegvesen.no/vegobjekter/105/",
	     // Path::new("./DATA/fartsgrenser"),
	     // "Fartsgrenseklient",
	     // &[("srid", "utm33")]
    // ).await;    
// }

fn get_next(val: &Value) -> String {
    match val["metadata"]["neste"]["href"].as_str() {
	Some(s) => s.to_owned(),
	None => panic!("{}", val),
    }
}

async fn requests(client: &Client, init_url: &str, dest: &Path, x_client: &str, params: &[(&str, &str)]) {
    let mut count = 0;
    let initial_request = match init_request(init_url, &client, x_client, params).await {
	Ok(o) => {
	    o.json::<Value>().await.unwrap()
	},
	Err(e) => panic!("{}", e),
    };

    let mut content = serde_json::to_string(&initial_request).unwrap();
    let mut filename = format!("{}.json", count);
    let mut filepath = dest.to_owned().to_path_buf();
    filepath.push(filename);
    
    let mut next_url = get_next(&initial_request);
    let file = File::create(filepath.clone()).unwrap();
    serde_json::to_writer(&file, &initial_request).unwrap();
    println!("Written {:?} to file", filepath);
    count += 1;

    let mut new_next_url;
    
    loop {
	let request = request(client, &next_url, x_client).await.unwrap().json::<Value>().await.unwrap();
	let mut filename = format!("{}.json", count);
	let mut filepath = dest.to_owned().to_path_buf();
	filepath.push(filename);

	new_next_url = get_next(&request);
	let file = File::create(filepath.clone()).unwrap();
	serde_json::to_writer(&file, &request).unwrap();
	println!("Written {:?} to file", filepath);
	count += 1;

	if next_url == new_next_url {
	    println!("Finished");
	    break;
	}
	else {
	    next_url = new_next_url;
	}
    }    
}
