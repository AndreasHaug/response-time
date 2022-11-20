import pymongo
import argparse

from pymongo import collection
from pymongo import mongo_client

import os

def parse_args():
    parser = argparse.ArgumentParser(description = "Extracting speedlimits data from NVDB speedlimit raw data")
    parser.add_argument("server", type = str, help = "MongoDB hostname or ip address")
    parser.add_argument("port", type = int, default = 27017, help = "port number of MongoDB instance")
    parser.add_argument("db_name", type = str, default = "roaddata", help = "name of database instance to use or create")
    parser.add_argument("db_rawspeedlimits_collection_name", type = str,
                        default = "raw_speedlimits",
                        help = "name of MongoDB collection storing the raw speedlimits data")
    parser.add_argument("speedlimit_collection_name", type = str, default = "speedlimits", help = "name of collection storing extracted speedlimit data from raw")
    return parser.parse_args()



    
def extract_speedlimits_from_raw(speedlimit_raw_collection: collection.Collection,
                                 speedlimit_collection: collection.Collection):
    
    def extract_speedlimit(val):
        def do_processing(inner):
            return {
                "id" : val["id"],
                "sequenze_id" : inner["veglenkesekvensid"],
                "startposition" : inner["startposisjon"],
                "endposition" : inner["sluttposisjon"],
                "speedlimit" : id_2021[0]["verdi"]
            }

        
        id_120105 = list(filter(lambda x: x["id"] == 120105, val["egenskaper"]))
        assert(len(id_120105) == 1)
        id_100105 = list(filter(lambda x: x["id"] == 100105, id_120105[0]["innhold"]))
        if len(id_100105) <= 0:
            return None
        id_2021 = list(filter(lambda x: x["id"] == 2021, val["egenskaper"]))
        assert(len(id_2021) == 1)
        return list(map(do_processing, id_100105))

    speedlimit_collection.drop()
    speedlimit_collection.drop_indexes()
    for val in speedlimit_raw_collection.find({}):
        limits = extract_speedlimit(val)
        if limits != None:
            speedlimit_collection.insert_many(limits)

            
    speedlimit_collection.create_index("seq_id")
    speedlimit_collection.create_index("id")
    speedlimit_collection.create_index("sequenze_id")
    speedlimit_collection.create_index("startposition")
    speedlimit_collection.create_index("endposition")

def main():
    args = parse_args()
    mongo_client = pymongo.MongoClient(args.server,
                                       args.port,
                                       username = os.environ.get("MONGO_USERNAME"),
                                       password = os.environ.get("MONGO_PASSWORD"),
                                       authSource = args.db_name,
                                       ssl = False)

    db = mongo_client[args.db_name]

    speedlimit_raw_collection = db[args.db_rawspeedlimits_collection_name]
    speedlimit_collection = db[args.speedlimit_collection_name]
    extract_speedlimits_from_raw(speedlimit_raw_collection, speedlimit_collection)
    
main()
