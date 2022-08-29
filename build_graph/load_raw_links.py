import os
import json
import argparse
import pymongo
from pymongo import mongo_client
from pymongo import collection
from pymongo.database import Database
import utm




def load_data(collection: collection.Collection, filepath: str):
    collection.drop()
    for name in os.listdir(filepath):
        file = open(os.path.join(filepath, name))
        val = json.load(open(os.path.join(filepath, name)))
        if len(val["objekter"]) != 0:
            collection.insert_many(val["objekter"])

        
def parse_args():
    parser = argparse.ArgumentParser(description = "Loading raw segmented link sequenzes from NVDB")
    parser.add_argument("server", type = str, help = "hostname or ip-address")
    # parser.add_argument("user", type = str, help = "MongoDB username")
    # parser.add_argument("passwd", type = str, help = "MongoDB password")
    parser.add_argument("rawlink_file_path", type = str, help = "path of folder containing json-data of segmented link sequenzes")
    parser.add_argument("rawspeedlimits_file_path", type = str, help = "path of folder containing json-data of speedlimits")
    parser.add_argument("port", type = int, default = 27017, help = "port number of MongoDB instance")
    parser.add_argument("db_name", type = str, default = "roaddata", help = "name of database instance to use or create")
    parser.add_argument("db_rawlink_collection_name", type = str, default = "raw_links", help = "name of MongoDB collection storing the raw roadlink data")
    parser.add_argument("db_rawspeedlimits_collection_name", type = str, default = "raw_speedlimits", help = "name of MongoDB collection storing the raw speedlimits data")
    return parser.parse_args()


def main():
    args = parse_args()
    mongo_client = pymongo.MongoClient(args.server,
                                       args.port,
                                       # username = args.user,
                                       # password = args.passwd,
                                       authSource = args.db_name,
                                       ssl = False)

    try:
        mongo_client.admin.command("ping")
        print("Database client was able to connect database")
    except:
        print("No connection to db: Exiting")
        return

    db: Database = mongo_client[args.db_name]
    rawlink_collection: collection.Collection = db[args.db_rawlink_collection_name]
    speedlimit_collection: collection.Collection = db[args.db_rawspeedlimits_collection_name]
    load_data(rawlink_collection, args.rawlink_file_path)
    load_data(speedlimit_collection, args.rawspeedlimits_file_path)
    

main()
