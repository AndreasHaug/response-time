use std::cmp::Ordering;
use std::collections::HashMap;
use std::vec::Vec;
use std::error::Error;
use crate::closest::Closest;
use crate::search::Search;
use geo::HaversineDistance;
use serde::{Serialize, Deserialize};


#[derive(Debug)]
pub struct Graph {
    nodes: HashMap<String, Node>,
    links: HashMap<String, RoadLink>,
}


const EARTH_RADIUS: f64 = 6371.088;


impl Graph {

    pub async fn new(nodes: HashMap<String, Node>, links: HashMap<String, RoadLink>) -> Self {
	Self {
	    nodes, links
	}
    }
    
    pub async fn search(&self, closest: Closest, cost: i32) -> Result<Search, Box<dyn Error>> {
	Search::do_search(&self, closest, cost).await
    }

    
    pub fn get_node(&self, id: &str) -> &Node {
	self.nodes.get(id).unwrap()
    }

    
    pub fn get_link(&self, reference: &str) -> &RoadLink {
	self.links.get(reference).unwrap()
    }

    
    pub async fn get_node_outlinks(&self, node: &Node) -> Vec<String> {
	node.links.iter()
	    .filter(|i| {
		let link = self.get_link(i);
		if node.id == link.startnode {
		    link.lanes.iter().any(|n| *n as i32 % 2 != 0) || link.lanes.is_empty()
		}
		else {
		    link.lanes.iter().any(|n| *n as i32 % 2 == 0) || link.lanes.is_empty()
		}
	    })
	    .map(|s| s.to_owned())
	    .collect()
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeCost {
    pub node: String,
    pub cost: i32,
}

impl NodeCost {
    pub fn new(node: String, cost: i32) -> Self {
	Self {
	    node, cost
	}
    }

    pub fn update_cost(&mut self, cost1: i32, cost2: i32) {
	self.cost = std::cmp::min(cost1, cost2);
    }
}


impl Ord for NodeCost {
    fn cmp(&self, other: &Self) -> Ordering {
	other.cost.cmp(&self.cost)
    }
}


impl PartialOrd for NodeCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
	other.cost.partial_cmp(&self.cost)
    }
}


impl PartialEq for NodeCost {
    fn eq(&self, other: &Self) -> bool {
	self.cost == other.cost
    }
}


impl Eq for NodeCost {}


#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: String,
    pub links: Vec<String>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct RoadLink {
    pub reference: String,
    pub seq_id: i32,
    pub startnode: String,
    pub endnode: String,
    pub startposition: f64,
    pub endposition: f64,
    pub length: i32,
    #[serde(default)]
    pub lanes: Vec<f32>,
    pub geometry: LineString,
    pub speedlimits: Vec<Speedlimit>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Speedlimit {
    id : i32,
    startposition: f64,
    endposition: f64,
    value: i8,
    super_placement: Option<SuperPlacement>,
}


impl Speedlimit {
    fn get_value(&self) -> i32 {
	self.value as i32
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SuperPlacement {
    #[serde(default)]
    lanes : Vec<i32>,
    
    startposition: f64,
    endposition: f64,
    seq_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LineString {
    r#type: String,
    pub coordinates: Vec<[f64; 2]>,
}


pub struct LineStringSegment<'a> {
    coordinates: &'a [[f64; 2]]
}


impl<'a> LineStringSegment<'a> {
    pub fn new(coords: &'a [[f64; 2]]) -> Self {
	LineStringSegment {
	    coordinates: coords,
	}
    }
}


pub struct PointPair {
    first: [f64; 2],
    last: [f64; 2],
}


impl PointPair {
    pub fn new(first: [f64; 2], last: [f64; 2]) -> Self {
	PointPair { first , last }
    }
}


pub trait Haversine {
    fn haversine_dist(&self) -> i32;
}


impl<'a> Haversine for LineStringSegment<'a> {
    fn haversine_dist(&self) -> i32 {
	let mut tot = 0;
	for a in 0..self.coordinates.len() - 1 {
	    let extra = PointPair::new(self.coordinates[a], self.coordinates[a + 1]).haversine_dist();
	    tot += extra;
	}
	tot
    }
}


impl Haversine for PointPair {

    fn haversine_dist(&self) -> i32 {

	let p1 = geo::Point::new(self.first[0], self.first[1]);
	let p2 = geo::Point::new(self.last[0], self.last[1]);
	f64::round(p1.haversine_distance(&p2)) as i32


	// let p1_lat = f64::to_radians(self.first[0]);
	// let p1_lng = f64::to_radians(self.first[1]);

	// let p2_lat = f64::to_radians(self.last[0]);
	// let p2_lng = f64::to_radians(self.last[1]);

	// let dlng = p2_lng - p1_lng;
	// let dlat = p2_lat - p1_lat;

	// let a = f64::powi(f64::sin(dlat / 2.0), 2) +
	    // f64::cos(p1_lat) * f64::cos(p2_lat) * f64::powi(f64::sin(dlng / 2.0), 2);

	// let c = 2.0 * f64::asin(f64::sqrt(a));
	// let earth_radius_m = EARTH_RADIUS * 1000.00;
	
	// f64::round(c * earth_radius_m) as i32
    }
}

impl Node {
    pub fn new(id: String, links: Vec<String>) -> Self {
	Self {id, links}
    }
}


impl RoadLink {

    pub fn startnode(&self) -> &String {
	&self.startnode
    }

    pub fn endnode(&self) -> &String {
	&self.endnode
    }

    pub fn get_destination_nodeid(&self, node: &Node) -> String {
	if node.id == self.startnode {
	    return self.endnode.to_owned();
	}
	else if node.id == self.endnode {
	    return self.startnode.to_owned();
	}
	else {
	    panic!("Node {} does not belong to link {}", node.id, self.reference);
	}
    }

    
    pub fn sub_linestring(&self, max_secs: i32, current_secs: i32, node_type: &str) -> Vec<[f64; 2]> {
	let mut dist = 0;
	let mut secs = current_secs;
	let mut substring: Vec<[f64; 2]> = Vec::new();
	let coordinates = self.geometry.coordinates.to_owned();

	let coordinates = match node_type {
	    "startnode" => coordinates,
	    _ => coordinates.into_iter().rev().collect::<Vec<[f64; 2]>>(),
	};
	
	for a in 0..coordinates.len() - 1 {
	    if secs > max_secs {
		break;
	    }
	    let pair = PointPair::new(coordinates[a], coordinates[a + 1]);
	    let added_dist = pair.haversine_dist();
	    dist += added_dist;
	    secs = self.driving_time_secs_on_given_length(dist);
	    substring.push(coordinates[a]);
	}
	substring
    }

    
    pub fn sub_linestring_from_to(&self,
				  max_secs: i32,
				  current_secs: i32,
				  node_type: &str,
				  from: usize,
				  to: usize) -> Vec<[f64; 2]> {
	
	let mut dist = 0;
	let mut secs = current_secs;
	let mut substring: Vec<[f64; 2]> = Vec::new();
	let coordinates = self.geometry.coordinates.to_owned();

	let coordinates = match node_type {
	    "startnode" => coordinates,
	    _ => coordinates.into_iter().rev().collect::<Vec<[f64; 2]>>(),
	};

	for a in from..to {
	    if secs > max_secs {
		break;
	    }
	    let pair = PointPair::new(coordinates[a], coordinates[a + 1]);
	    let added_dist = pair.haversine_dist();
	    dist += added_dist;
	    // println!("dist: {}", dist);
	    secs = self.driving_time_secs_on_given_length(dist);
	    substring.push(coordinates[a]);
	}
	substring
    }


    #[inline]
    pub fn speedlimits_values(&self) -> impl Iterator<Item = i32> + '_ {
	self.speedlimits.iter().map(|s| s.get_value())
    }

    #[inline]
    fn avg_km_per_h(&self) -> f64 {
	match self.speedlimits.is_empty() {
	    true => 50.00,
	    _ => (self.speedlimits_values().sum::<i32>() / self.speedlimits.len() as i32) as f64,
	}
    }

    
    #[inline]
    pub fn driving_time_secs(&self) -> i32 {
	let meters_per_second: f64 = f64::round(self.get_estimated_driving_speed() * 1000.00 / 60.00 / 60.00);
	f64::round(self.length as f64 / meters_per_second  * self.driving_time_add_factor()) as i32
    }

    
    #[inline]
    pub fn driving_time_secs_on_given_length(&self, length: i32) -> i32 {
	let meters_per_second: f64 = f64::round(self.get_estimated_driving_speed() * 1000.00 / 60.00 / 60.00);
	f64::round(length as f64 / meters_per_second  * self.driving_time_add_factor()) as i32
    }


    #[inline]
    pub fn driving_time_add_factor(&self) -> f64 {
	let avg_km_per_h = self.avg_km_per_h();
	if avg_km_per_h <= 40.00 {
	    return 1.5
	}
	if avg_km_per_h <= 50.00 {
	    return 1.45
	}
	if avg_km_per_h <= 60.00 {
	    return 1.40
	}
	if avg_km_per_h <=  70.00 {
	    return 1.35
	}
	if avg_km_per_h <= 80.00 {
	    return 1.30
	}
	if avg_km_per_h <= 90.00 {
	    return 1.20
	}
	if avg_km_per_h <= 100.00 {
	    return 1.15
	}
	if avg_km_per_h <=  110.00 {
	    return 1.1
	}
	1.1
    }


    #[inline]
    pub fn get_estimated_driving_speed(&self) -> f64 {
	let avg_km_per_h = self.avg_km_per_h();
	if avg_km_per_h <= 40.00 {
	    return avg_km_per_h + 5.00;
	}
	if avg_km_per_h <= 50.00 {
	    return avg_km_per_h + 10.00;
	}
	if avg_km_per_h <= 60.00 {
	    return avg_km_per_h + 15.00;
	}
	if avg_km_per_h <=  70.00 {
	    return avg_km_per_h + 20.00;
	}
	if avg_km_per_h <= 80.00 {
	    return avg_km_per_h + 25.00;
	}
	if avg_km_per_h <= 90.00 {
	    return avg_km_per_h + 30.00;
	}
	if avg_km_per_h <= 100.00 {
	    return avg_km_per_h + 40.00;
	}
	if avg_km_per_h <=  110.00 {
	    return avg_km_per_h + 50.00;
	}
	avg_km_per_h
    }
}
