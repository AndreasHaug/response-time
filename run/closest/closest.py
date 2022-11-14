import os
import pymongo
import argparse
from pymongo import mongo_client
from pymongo import collection
from fastapi import FastAPI, Request, Response
import uvicorn
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
from uvicorn.main import print_version



print("starting closest")
print(os.environ.get("MONGO_USERNAME"))
mongo_client: pymongo.MongoClient = pymongo.MongoClient(
    # "localhost",
    # "database",
    os.environ.get("MONGO_DB_CONNECTION"),
    27017,
    username = os.environ.get("MONGO_USERNAME"),
    password = os.environ.get("MONGO_PASSWORD"),
    authSource = "roaddata",
    ssl = False
)


db = mongo_client["roaddata"]
link_collection: collection.Collection = db["links"]
points_collection: collection.Collection = db["points"]
app = FastAPI()
root = os.path.dirname(os.path.abspath(__file__))


def parse_args():
    parser = argparse.ArgumentParser(description = "Finding closest point in the road")


def closest_link(lat: float, lng: float):
    ret = link_collection.find_one(
        {
	    "geometry" :
	    {
	        "$nearSphere":
	        {
		    "$geometry":
		    {
		        "type": "LineString",
		        "coordinates": [lng, lat]
		    },
		    "$minDistance": 0,
		    "$maxDistance": 500
	        }
	    }
        },
        { "_id" : 0, "reference" : 1 }
    )
    return ret["reference"]


@app.get("/")
async def closest(lat: float, lng: float):
    cl = closest_link(lat, lng)
    if cl == None:
        return cl
    
    res = points_collection.find_one(        
        {
            "link" : cl,
	    "geometry" :
	    {
	        "$nearSphere":
	        {
		    "$geometry":
		    {
		        "type": "Point",
		        "coordinates": [lng, lat]
		    },
		    "$minDistance": 0,
		    "$maxDistance": 500
	        }
	    }
        },
        { "_id" : 0 }
    )
    res["geometry"]["coordinates"].reverse()
    return res


@app.get("/closest_point")
async def closest_point(lat: float, lng: float):
    res = points_collection.find_one(
        {
	    "geometry" :
	    {
	        "$nearSphere":
	        {
		    "$geometry":
		    {
		        "type": "Point",
		        "coordinates": [lng, lat]
		    },
		    "$minDistance": 0,
		    "$maxDistance": 500
	        }
	    }
        },
        { "_id" : 0 }
    )
    return res


if __name__ == "__main__":
    uvicorn.run(app, host = "0.0.0.0", port = 8001)
