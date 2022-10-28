import argparse
import pymongo
from pymongo import collection
import utm


def utm_to_latlng(north: float, east: float) -> list[float]:
    latitude, longitude = utm.to_latlng(east, north, 33, northern = True, strict = False)
    return [longitude, latitude]
    

def load_roadlinks_from_raw(raw_link_collection: collection.Collection,
                            link_collection: collection.Collection,
                            speedlimit_collection: collection.Collection):
    for a in filter(filter_detaillevel_and_type, raw_link_collection.find({},
                                      {
                                          "referanse" : 1,
                                          "veglenkesekvensid" : 1,
                                          "startnode" : 1,
                                          "sluttnode" : 1,
                                          "startposisjon" : 1,
                                          "sluttposisjon" : 1,
                                          "lengde" : 1,
                                          "feltoversikt" : 1,
                                          "geometri" : 1,
                                          "superstedfesting" : 1,
                                          "typeVeg_sosi" : 1,
                                          "detaljnivå" : 1
                                      })):
        
        if a.get("superstedfesting") == None:
            link = {
                "reference" : a["referanse"],
                "seq_id" : a["veglenkesekvensid"],
                "startnode" : a["startnode"],
                "endnode" : a["sluttnode"],
                "startposition" : a["startposisjon"],
                "endposition" : a["sluttposisjon"],
                "length" : round(a["lengde"]),
                "lanes" : extract_lane_numbers(a["feltoversikt"]) if a.get("feltoversikt") != None else [],
                "geometry" : geometry_as_geojson_latlng(a),
                "speedlimits" : attach_speedlimits(a, speedlimit_collection)
            }
        else:
            super_placement = a["superstedfesting"]
            
            link =  {
                "reference" : a["referanse"],
                "seq_id" : a["veglenkesekvensid"],
                "startnode" : a["startnode"],
                "endnode" : a["sluttnode"],
                "startposition" : a["startposisjon"],
                "endposition" : a["sluttposisjon"],
                "length" : round(a["lengde"]),
                "lanes" : extract_lane_numbers(super_placement["kjørefelt"]),
                "geometry" : geometry_as_geojson_latlng(a),
                # "super_placement" : a["superstedfesting"],
                # "super_placement" : super_placement,
                "speedlimits" : attach_speedlimits(a, speedlimit_collection)
            }
        link_collection.insert_one(link)
    #endif
        
    link_collection.create_index([("geometry", pymongo.GEOSPHERE)])
    link_collection.create_index("startnode")
    link_collection.create_index("endnode")
    link_collection.create_index("reference")
    link_collection.create_index("seq_id")
    link_collection.create_index("startposition")
    link_collection.create_index("endposition")


def extract_lane_numbers(lanes: list[str]) -> list[int]:
    lanes_ints: list[int] = []
    for a in lanes:
        if len(a) == 1:
            lanes_ints.append(int(a))
        else:
            if a[1].isnumeric():
                lanes_ints.append(int(a[0:2]))
        # end if
    # end for
    return lanes_ints
# end extract_lane_numbers
            
              
def utm33_to_latlng(north, east) -> list[float]:
    latitude, longitude = utm.to_latlon(east, north, 33, northern = True, strict = False)
    return [longitude, latitude]


def geo_json_points(val) -> list[list[float]]:
    start = 13 if val.startswith("LINESTRING Z") else 11
    return list(map(lambda x: utm33_to_latlng(float(x[1]), float(x[0])),
                    map(lambda s: s.split(" "), val[start::].split(", "))))


def geometry_as_geojson_latlng(val) -> dict:
    return {
        "type" : "LineString", "coordinates" : geo_json_points(val["geometri"]["wkt"])
    }
        

def find_link_speedlimits_range(link_startpos, link_endpos, limit_min_pos, limit_max_pos):
    if limit_max_pos <= link_startpos or limit_min_pos >= link_endpos:
        return None
    link_speedlimit_min = max(limit_min_pos, link_startpos)
    link_speedlimit_max = min(limit_max_pos, link_endpos)
    return link_speedlimit_min, link_speedlimit_max
    

def attach_speedlimits(roadlink, speedlimit_collection):
    if roadlink.get("superstedfesting") == None:
        seq_id = roadlink["veglenkesekvensid"]
        link_startpos = roadlink["startposisjon"]
        link_endpos = roadlink["sluttposisjon"]
    else:
        seq_id = roadlink["superstedfesting"]["veglenkesekvensid"]
        link_startpos = roadlink["superstedfesting"]["startposisjon"]
        link_endpos = roadlink["superstedfesting"]["sluttposisjon"]

    speedlimits = speedlimit_collection.find({ "sequenze_id" : seq_id })
    
    link_speedlimits = []
    for s in speedlimits:
        value = s["speedlimit"]
        limit_start_pos = s["startposition"]
        limit_end_pos = s["endposition"]
        limit_min_pos = min(limit_start_pos, limit_end_pos)
        limit_max_pos = max(limit_start_pos, limit_end_pos)

        limit_range = find_link_speedlimits_range(link_startpos, link_endpos, limit_min_pos, limit_max_pos)
        if limit_range == None:
            continue
        if roadlink.get("superstedfesting") == None:
            speedlimit = { "id" : s["id"],
                           "startposition" : limit_range[0],
                           "endposition" : limit_range[1],
                           "value" : value }
        else:
            speedlimit = { "id" : s["id"],
                           "startposition" : limit_range[0],
                           "endposition" : limit_range[1],
                           "value" : value,
                           "super_placement" : {
                               "seq_id" : seq_id,
                               "startposition" : limit_range[0],
                               "endposition" : limit_range[1],
                           }}
        link_speedlimits.append(speedlimit)
    # print()
    return link_speedlimits

    
def load_nodes(raw_link_collection: collection.Collection, node_collection: collection.Collection):
    print("Getting node ids")
    node_ids = load_node_ids(raw_link_collection) 
    print("Finished reading node ids")
    print("Extracting nodes")
    for a in node_ids:
        links = list(map(lambda x: x.get("reference"),
                         raw_link_collection.find({ "$or" :
                                                    [{ "startnode" : a }, { "sluttnode" : a }]
                                                   },
                                                  { "reference" : 1 })))
        node_collection.insert_one({
            "id" : a,
            "links" : links
        })
        

def load_node_ids(raw_link_collection) -> set[str]:
    node_ids: set[str] = set()
    for a in filter(filter_detaillevel_and_type, raw_link_collection.find({},
                                                                          {"startnode" : 1,
                                                                           "sluttnode" : 1,
                                                                           "typeVeg_sosi" : 1,
                                                                           "detaljnivå" : 1})):
        node_ids.add(a["startnode"])
        node_ids.add(a["sluttnode"])
               
    return node_ids


def filter_detaillevel_and_type(val) -> bool:
    type_road: str = val["typeVeg_sosi"]
    return (
        (type_road == "enkelBilveg" or
         type_road == "kanalisertVeg" or
         type_road == "rampe" or
         type_road == "rundkjøring" or
         type_road == "gatetun")
        and
        val["detaljnivå"] != "Vegtrase")


def load_points(raw_link_collection: collection.Collection, points_collection: collection.Collection):


    def get_node(val, i, length):
        if i == 0:
            return val["startnode"]
        elif i == length - 1:
            return val["sluttnode"]
        return None
    
    for a in filter(filter_detaillevel_and_type,
                    raw_link_collection.find({},
                                             {
                                                 "referanse" : 1,
                                                 "geometri" : 1,
                                                 "startnode" : 1,
                                                 "sluttnode" : 1,
                                                 "typeVeg_sosi" : 1,
                                                 "detaljnivå" : 1
                                             })):

        geometry = geometry_as_geojson_latlng(a)
        coordinates_length = len(geometry["coordinates"])
        for i, b in enumerate(geometry):
            node = get_node(a, i, coordinates_length)
            t = {
                "link" : a["referanse"],
                "geometry" : {
                    "type" : "Point",
                    "coordinates" : geometry["coordinates"][i],
                },
                "node" : node,
                "linestring_index" : i,
                "linestring_length" : coordinates_length
            }
            points_collection.insert_one(t)
                    
    points_collection.create_index([("geometry", pymongo.GEOSPHERE)])



def load_from_raw(raw_link_collection: collection.Collection,
               link_collection: collection.Collection,
               speedlimit_collection: collection.Collection,
               node_collection: collection.Collection,
               points_collection: collection.Collection):
    
    print("loading links")
    link_collection.drop()
    link_collection.drop_indexes()
    load_roadlinks_from_raw(raw_link_collection, link_collection, speedlimit_collection)
    print("loaded links")

    
    print("loading nodes")
    node_collection.drop()
    node_collection.drop_indexes()
    load_nodes(raw_link_collection, node_collection)
    print("loaded nodes")

    print("loading points")
    points_collection.drop()
    points_collection.drop_indexes()
    load_points(raw_link_collection, points_collection)
    print("loaded points")
    


def parse_args():
    parser = argparse.ArgumentParser(description = "Loading raw segmented link sequenzes from NVDB")
    parser.add_argument("server", type = str, help = "hostname or ip-address")
    # parser.add_argument("user", type = str, help = "MongoDB username")
    # parser.add_argument("passwd", type = str, help = "MongoDB password")
    parser.add_argument("port", type = int, default = 27017, help = "port number of MongoDB instance")
    parser.add_argument("db_name", type = str, default = "roaddata", help = "name of database instance to use or create")
    parser.add_argument("raw_link_collection_name", type = str, default = "raw_links", help = "name of collection storing raw links")
    parser.add_argument("link_collection_name", type = str, default = "links", help = "name of collection storing road links")
    parser.add_argument("speedlimit_collection_name", type = str, default = "speedlimits", help = "name of collection storing extracted speedlimit data")
    parser.add_argument("node_collection_name", type = str, default = "nodes", help = "name of collection to store nodes")
    parser.add_argument("points_collection_name", type = str, default = "points", help = "name of collection to store points")
    
    return parser.parse_args()    


def main():
    args = parse_args()
    mongo_client: pymongo.MongoClient = pymongo.MongoClient(args.server,
                                       args.port,
                                       # username = args.username,
                                       # password = args.passwd,
                                       authSource = args.db_name,
                                       ssl = False)

    
    db = mongo_client[args.db_name]
    raw_link_collection = db[args.raw_link_collection_name]
    link_collection = db[args.link_collection_name]
    speedlimit_collection = db[args.speedlimit_collection_name]
    node_collection = db[args.node_collection_name]
    points_collection = db[args.points_collection_name]
    
    load_from_raw(raw_link_collection, link_collection, speedlimit_collection, node_collection, points_collection)
    
main()
