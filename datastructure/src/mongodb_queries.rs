use std::vec;

use crate::closest::Closest;
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

#[inline]
pub(crate) fn extract_from_rawlink() -> Vec<Document> {
    vec![
	doc! {
	    "$match" : {
		"kommune" : 301
		// "kommune" : 1856
	    }
	},	
	doc! {
	    "$match" : {
		"$or" : [
		    // {
		    // 	"typeVeg" : "Kanalisert veg"
		    // },
		    // {
		    // 	"typeVeg" :  "Enkel bilveg"
		    // },
		    // {
		    // 	"typeVeg" : "Kanalisert veg"
		    // },
		    // {
		    // 	"typeVeg" :  "Enkel bilveg"
		    // },
		    // {
		    // 	"typeVeg" : "Rampe",
		    // },
		    // {
		    // 	"typeVeg" : "Rundkjøring",
		    // },
		    // {
		    // 	"typeVeg" : "Gatetun",
		    // },
		    {
			"detaljnivå" : "Vegtrase og kjørebane"
		    },
		    {
			"detaljnivå" : "Kjørebane"
		    },
		    		    {
			"detaljnivå" : "Kjørefelt"
		    }
		]
	    }
	},
	doc! {
    "$project" :
    doc!{
    "reference" : "$referanse",
    "startnode" : "$startnode",
    "endnode" : "$sluttnode",
    "length" : doc!{ "$convert" :
             {
                 "input" : doc!{ "$round" : "$lengde"},
                 "to" : "int",
             },
               },
    "lanes" : {
        "$function" : {
        "body" : "function(s, t, f) {
			let r;
			if (t == \"HOVED\") {
			    r = f;
			}
			else if (f !== null) {
			    r = s;
			}
			else {
			    r = []
			}
			
			fieldNumbers = [];
			for (let a in r) {
			    field_number = \"\";
			    for (let b in r[a]) {
				if (!isNaN(r[a][b])) {
				    field_number += r[a][b]
				}
				else {
				    fieldNumbers.push(parseInt(field_number));
				    break;
				}
			    }
			}
			return fieldNumbers;
		    }"
            ,
            "args" : ["$superstedfesting.kjørefelt", "$type", "$feltoversikt"],
            "lang" : "js"
        }
    },
    "geometry" : {
        "$function" : {
        "body" : "function(s) {
			    let startIndex = s[11] === \"Z\" ? 13 : 11
				return { \"type\" : \"LineString\",
					  \"coordinates\" : s.slice(startIndex, -1)
					  .split(\", \")
					  .map(r => r.split(\" \"))
					  .map(p => [p[0], p[1]]
					       .map(parseFloat)),
				}
			}",
        "args": ["$geometri.wkt"],
        "lang" : "js",
        }
    }
    }}]
}

//the haversine formula as is at https://www.geeksforgeeks.org/haversine-formula-to-find-distance-between-two-points-on-a-sphere/
pub fn closest_link(lat: f64, lon: f64) -> Vec<Document> {
    vec![
        doc! {
        "$geoNear" : {
            "near" : { "type" : "Point", "coordinates" : [lon, lat] },
            "distanceField" : "dist.calculated",
            "maxDistance" : 10000,
            "spherical" : true,
        }
        },
        doc! {
            "$project" : {
		// "distanceField" : "$distanceField",
		
            "reference" : "$reference",
            "linestring_index" : {
                "$toInt" : {
                "$function" : {
                "body": "function(lat, lon, c, n) {
			let lat1 = lat;
			let lon1 = lon;
			function haversine(lat1, lon1, lat2, lon2) {
			    let dLat = (lat2 - lat1) * Math.PI / 180.0;
			    let dLon = (lon2 - lon1) * Math.PI / 180.0;
			    lat1 = (lat1) * Math.PI / 180.0;
			    lat2 = (lat2) * Math.PI / 180.0;
			    let a = Math.pow(Math.sin(dLat / 2), 2) +
				Math.pow(Math.sin(dLon / 2), 2) *
				Math.cos(lat1) *
				Math.cos(lat2);
			    let rad = 6371;
			    let c = 2 * Math.asin(Math.sqrt(a));
			    return rad * c;
			}
			let dists = c.map(t => haversine(lat1, lon1, t[1], t[0]))
			return dists.indexOf(Math.min(...dists))
		    }",
                "args" : [lat, lon, "$geometry.coordinates", 1],
                "lang" : "js"
                }
                }
            },
            "linestring_length" : {
                "$toInt" : { "$size" : "$geometry.coordinates" }
            }
		// "start" : { "$arrayElemAt" : ["$geometry.coordinates", "$linestring_index"]}
            }
        },
        doc! {
            "$limit" : 1
        },
    ]
}

// fn extract_lanes() {
//     doc! {
// 	"$function" : {
// 	    "body" : "function(s, t, f) {
// 			let r;
// 			if (t == \"HOVED\") {
// 			    r = f;
// 			}
// 			else if (f !== null) {
// 			    r = s;
// 			}
// 			else {
// 			    r = []
// 			}

// 			fieldNumbers = [];
// 			for (let a in r) {
// 			    field_number = \"\";
// 			    for (let b in r[a]) {
// 				if (!isNaN(r[a][b])) {
// 				    field_number += r[a][b]
// 				}
// 				else {
// 				    fieldNumbers.push(parseInt(field_number));
// 				    break;
// 				}
// 			    }
// 			}
// 			return fieldNumbers;
// 		    }"
// 		,
// 	    "args" : ["$superstedfesting.kjørefelt", "$type", "$feltoversikt"],
// 	    "lang" : "js"
// 	}
//     }
// }
