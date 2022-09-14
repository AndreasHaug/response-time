use std::collections::{HashSet, BinaryHeap, HashMap};
use geo::{MultiLineString};
use geo::ConvexHull;
use serde::Serialize;
use std::error::Error;
use crate::closest::Closest;
use crate::graph::{Graph, Node, LineStringSegment, NodeCost, RoadLink};
use crate::search_result::{MultiLineStringResult, PolygonResult, SearchResult};
use crate::graph::Haversine;
// use rs_concaveman::{concaveman, location_trait::LocationTrait};

pub struct Search<'a> {
    start: Start,
    utils: SearchUtils,
    pub linestrings: Vec<Vec<[f64; 2]>>,
    closest_link: Option<Closest>,
    graph: Option<&'a Graph>,
    cost: i32,
}


pub struct SearchUtils {
    explored: HashSet<String>,
    queue: BinaryHeap<NodeCost>,
    node_costs: HashMap<String, NodeCost>,
}

#[derive(Serialize, Debug)]
pub struct Start {
    r#type : String,
    coordinates: [f64; 2],
}


impl<'a> Search<'a> {
    
    pub async fn new(start: Start, graph: &'a Graph, closest: Closest, cost: i32) -> Search<'a> {
	Self {
	    start,
	    utils: SearchUtils::new().await,
	    linestrings: Vec::new(),
	    closest_link: Some(closest),
	    graph: Some(graph),
	    cost,	   
	}
    }
    
    pub fn as_multilinestring(self) -> SearchResult {
	SearchResult::new(
	    self.start,
	    crate::search_result::SearchResultProperty::MultilinestringResult(MultiLineStringResult::new(self.linestrings))
	)
    }

    
    pub fn as_polygon(self) -> SearchResult {

	let polygon = PolygonResult::new(	    
	    MultiLineString::from_iter(
		self.linestrings
		    .into_iter()
		    .map(geo::LineString::from))
		.convex_hull()
		.exterior()
		.clone()
		.into_points()
		.iter()
		.map(|p|[p.x(), p.y()])
		.collect::<Vec<[f64; 2]>>());

	SearchResult::new(
	    self.start,
	    crate::search_result::SearchResultProperty::PolygonResult(polygon)
	)
    }

    async fn closest(&self) -> Option<&Closest> {
	self.closest_link.as_ref()
    }

    
    pub async fn do_search(graph: &'a Graph, closest: Closest, cost: i32) -> Result<Search, Box<dyn Error>>{
	let cost_seconds = cost * 60;
	let node: bool = closest.node.is_some();
	
	let mut search = Search::new(Start::new(closest.geometry.coordinates),
				     graph,
				     closest,
				     cost_seconds).await;
	if node {
	    search.do_search_from_node().await;
	}
	else {
	    search.do_search_on_link().await;
	}
	Ok(search)
    }
    
    pub async fn do_search_from_node(&mut self) {
	let startnodeid = self.closest().await.unwrap().node().unwrap();
	self.utils.add_cost(startnodeid.to_owned(), 0).await;
	while let Some(s) = self.utils.queue.pop() {
	    self.explore_node(&s.node).await;
	}
    }

    
    pub async fn do_search_on_link(&mut self) {
	let link = self.graph.unwrap().get_link((self.closest().await).unwrap().link());
	let index = self.closest().await.unwrap().linestring_index as usize;
	
	if link.lanes.iter().any(|n| *n as i32 % 2 != 0) || link.lanes.is_empty() {
	    let dist_to_endnode = LineStringSegment::new(&link
							 .geometry
							 .coordinates[index..link.geometry.coordinates.len()]).haversine_dist();
	    
	    let meters_per_second: f64 = f64::round(link.get_estimated_driving_speed() * 1000.00 / 60.00 / 60.00);
	    let driving_time = f64::round(dist_to_endnode as f64 / meters_per_second  * link.driving_time_add_factor()) as i32;
	    
	    
	    if driving_time > self.cost {
		let sublinestring: Vec<[f64; 2]> = link.sub_linestring_from_to(self.cost, 0, "startnode", index, link.geometry.coordinates.len());
		self.linestrings.push(sublinestring);
	    }
	    else {			    
		self.linestrings.push(link.geometry.coordinates[index..link.geometry.coordinates.len()].to_vec());
		self.utils.add_cost(link.endnode.to_owned(), driving_time).await;
	    }
	}
	
	if link.lanes.iter().any(|n| *n as i32 % 2 == 0) || link.lanes.is_empty() {
	    let dist_to_startnode = LineStringSegment::new(&link.geometry.coordinates[0..index]).haversine_dist();

	    let meters_per_second: f64 = f64::round(link.get_estimated_driving_speed() * 1000.00 / 60.00 / 60.00);
	    let driving_time = f64::round(dist_to_startnode as f64 / meters_per_second  * link.driving_time_add_factor()) as i32;
	    
	    if driving_time > self.cost {
		let sublinestring: Vec<[f64; 2]> = link.sub_linestring_from_to(self.cost, 0, "endnode", 0, index + 1);
		self.linestrings.push(sublinestring);
	    }
	    else {
		self.linestrings.push(link
				      .geometry
				      .coordinates[0..index + 1].to_vec());
		
		self.utils.add_cost(link.startnode.to_owned(), driving_time).await;
	    }
	}				
		
	while let Some(s) = self.utils.queue.pop() {
	    self.explore_node(&s.node).await;
	}
    }

    
    #[inline]
    async fn get_node_outlinks(&self, node: &Node) -> Vec<String> {
	self.graph.unwrap().get_node_outlinks(node).await
    }


    #[inline]
    async fn explore_node(&mut self, origin_node_id: &str) {
	let origin_node_cost: i32 = self.utils.get_cost_value(origin_node_id).await.unwrap();
	let graph = self.graph.unwrap();
	let origin_node: &Node = graph.get_node(origin_node_id);
	
	
	for linkid in self.get_node_outlinks(origin_node).await {
	    let link: &RoadLink = graph.get_link(&linkid);
	    let node_type;
	    if origin_node.id == link.startnode {
		node_type = "startnode";
	    }
	    else {
		node_type = "endnode";
	    }


	    let driving_time = link.driving_time_secs();
	    let destination_nodeid = link.get_destination_nodeid(origin_node);
	    let destination_cost = origin_node_cost + driving_time;
	    if destination_cost > self.cost {
		let sublinestring = link.sub_linestring(self.cost, origin_node_cost, node_type);
		if !sublinestring.is_empty() {
		    self.linestrings.push(sublinestring);
		}
		continue;
	    }
	    else {
		self.linestrings.push(link.geometry.coordinates.clone());
	    }
	    
	    if !self.utils.is_explored(&destination_nodeid).await {
		match self.utils.get_cost_value(&destination_nodeid).await {
		    Some(s) => self.utils.update_to_min_cost(&destination_nodeid,
							    s,
							    origin_node_cost + driving_time).await,
		    None => self.utils.add_cost(destination_nodeid,
					       origin_node_cost + driving_time).await,
		}
		self.utils.set_explored(origin_node_id).await;
	    }
	}
    }
}


impl SearchUtils {
    async fn new() -> Self {
	Self {
    	    explored: HashSet::new(),
	    queue: BinaryHeap::new(),
	    node_costs: HashMap::new(),
	}
    }

    
    async fn set_explored(&mut self, nodeid: &str) {
	self.explored.insert(nodeid.to_owned());
    }

    
    async fn is_explored(&self, nodeid: &str) -> bool {
	self.explored.get(nodeid).is_some()
    }

    
    async fn add_cost(&mut self, nodeid: String, cost: i32) {
	let queue_entry = NodeCost::new(nodeid.to_owned(), cost);
	let node_costs_entry = queue_entry.clone();
	self.queue.push(queue_entry);
	self.node_costs.insert(nodeid, node_costs_entry);
    }


    async fn get_cost_value(&self, nodeid: &str) -> Option<i32> {
	if let Some(s) = self.node_costs.get(nodeid) {
	    return Some(s.cost)
	}
	None
    }

    
    async fn update_to_min_cost(&mut self, nodeid: &str, cost1: i32, cost2: i32) {
	self.node_costs.get_mut(nodeid)
	    .unwrap()
	    .update_cost(cost1, cost2);
    }
}


impl Start {
    pub fn new(coordinates: [f64; 2]) -> Self {
	Start {
	    r#type: String::from("Point"),
	    coordinates,
	}	
    }
}
