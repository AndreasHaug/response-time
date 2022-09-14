use std::vec;

use mongodb::bson::{doc, Document};

pub(crate) fn get_links() -> Vec<Document> {
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
