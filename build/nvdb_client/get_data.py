import requests
import json
from pathlib import Path
import time
import os
import sys

VERSION = 0.1
DEFAULT_LOCATION=Path("./DATA")



def get(url, x_client, dest, params = {}):
    print("Getting", dest)
    headers = {
        "Accept" : "application/vnd.vegvesen.nvdb-v3-rev1+json",
        "X-Client" : x_client,
    }

    count = 0
    next = url

    if not os.path.exists(os.path.join(DEFAULT_LOCATION, dest)):
        os.makedirs(os.path.join(DEFAULT_LOCATION, dest))
    
    while next != None:
        if count != 0:
            params = {}
        
        response = requests.get(next, headers=headers, params=params)
        data = response.json()
        if len(data) != 2:
            print("error")
            os.exit(1)
            
        n_next = data["metadata"]["neste"]["href"]
        if n_next == next:
            return
        else:
            next = n_next
        
        target = DEFAULT_LOCATION / dest / str(count)
        with open(target, "w") as fd:
            fd.write(json.dumps(data))
            print(count, "written")
        count += 1
        # time.sleep(0.5)
    
def main():


    get('https://nvdbapiles-v3.atlas.vegvesen.no/vegobjekter/105',
        "Fartsgrenseklient",
        "fartsgrenser",
        { 'inkluder' : 'egenskaper',
          'srid' : 'wgs84',
          
          # for making selection of Oslo and Viken
          # comment out "fylke" : "3,30" to get data of whole country
          "fylke" : "3,30",
         })


    get("https://nvdbapiles-v3.atlas.vegvesen.no/vegnett/veglenkesekvenser/segmentert/",
        "Vegnettklient",
        "veglenkesekvenser",
        { 'srid' : 'utm33',

          # for making selection of Oslo and Viken
          # comment out "fylke" : "3,30" to get data of whole country 
          "fylke" : "3,30"
         })
main()
