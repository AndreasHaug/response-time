use std::collections::{HashSet, BinaryHeap, HashMap};
use geo::{MultiLineString};
use geo::ConvexHull;
use std::error::Error;
use crate::closest::Closest;
use crate::graph::{Graph, Node, LineStringSegment, NodeCost, RoadLink};
use crate::search_result::{MultiLineStringResult, PolygonResult, SearchResult, Start};
use crate::graph::Haversine;


pub struct Search<'a> {
    utils: SearchUtils,
    pub linestrings: Vec<Vec<[f64; 2]>>,
    closest_link: Closest,
    graph: &'a Graph,
    cost: i32,
}


pub struct SearchUtils {
    explored: HashSet<String>,
    explored_links: HashSet<String>,
    queue: BinaryHeap<NodeCost>,
    node_costs: HashMap<String, NodeCost>,
}


impl<'a> Search<'a> {
    
    pub async fn new(graph: &'a Graph, closest: Closest, cost: i32) -> Search<'a> {
	Self {
	    // start,
	    utils: SearchUtils::new().await,
	    linestrings: Vec::new(),
	    // closest_link: Some(closest),
	    closest_link: closest,
	    graph: graph,
	    cost,	   
	}
    }
    
    pub fn as_multilinestring(self) -> SearchResult {
	SearchResult::new(
	    // self.start,
	   Start::new(self.closest_link.geometry.coordinates),
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
	    Start::new(self.closest_link.geometry.coordinates),
	    crate::search_result::SearchResultProperty::PolygonResult(polygon)
	)
    }

    
    async fn closest(&self) -> &Closest {
	&self.closest_link
    }

    
    pub async fn do_search(graph: &'a Graph, closest: Closest, cost: i32) -> Result<Search, Box<dyn Error>>{
	let cost_seconds = cost * 60;
	let node: bool = closest.node.is_some();
	
	let mut search = Search::new(
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
	let startnodeid = self.closest().await.node().unwrap();
	self.utils.add_cost(startnodeid.to_owned(), 0).await;
	while let Some(s) = self.utils.queue.pop() {
	    self.explore_node(&s.node).await;
	}
    }

    
    pub async fn do_search_on_link(&mut self) {
	let link = self.graph.get_link((self.closest().await).link());
	let index = self.closest().await.linestring_index as usize;
	
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
	self.graph.get_node_outlinks(node).await
    }


    #[inline]
    async fn explore_node(&mut self, origin_node_id: &str) {
	if self.utils.is_explored(origin_node_id).await {
	    return;
	}
	let origin_node_cost: i32 = self.utils.get_cost_value(origin_node_id).await.unwrap();
	let graph = self.graph;
	let origin_node: &Node = graph.get_node(origin_node_id);

	
	
	
	for linkid in self.get_node_outlinks(origin_node).await {

	    let link: &RoadLink = graph.get_link(&linkid);
	    let node_type;
	    if origin_node.id == link.startnode {
		node_type = "startnode";
	    }
	    else if origin_node.id == link.endnode {
		node_type = "endnode";
	    }
	    else {
		panic!("Origin node  {}", origin_node_id);
	    }


	    let driving_time = link.driving_time_secs();
	    let destination_nodeid = link.get_destination_nodeid(origin_node);
	 if !self.utils.is_link_explored(&link.reference).await {
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
		self.utils.set_link_explored(&link.reference).await;
		
	    }
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
	    explored_links: HashSet::new(),
	    queue: BinaryHeap::new(),
	    node_costs: HashMap::new(),
	}
    }

    
    async fn set_explored(&mut self, nodeid: &str) {
	self.explored.insert(nodeid.to_owned());
    }

    
    async fn set_link_explored(&mut self, link_reference: &str) {
	self.explored_links.insert(link_reference.to_owned());
    }

    
    async fn is_explored(&self, nodeid: &str) -> bool {
	self.explored.get(nodeid).is_some()
    }

    async fn is_link_explored(&self, link_reference: &str) -> bool {
	self.explored_links.get(link_reference).is_some()
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

    
    async fn update_to_min_cost(&mut self, nodeid: &str, cost: i32, possible_new_cost: i32) {
	self.node_costs.get_mut(nodeid)
	    .unwrap()
	    .update_cost(cost, possible_new_cost);

	if possible_new_cost < cost {
	    self.queue.push(NodeCost::new(nodeid.to_owned(), cost));
	}
	
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
