from neo4j import Driver, GraphDatabase, Session
import neo4j
from neo4j.exceptions import ClientError
import pymongo
from pymongo import MongoClient, collection
import itertools


def insert_nodes(driver: neo4j.Driver, nodecollection: collection.Collection):
    session: Session = driver.session()
    nodes = nodecollection.find({}, {"id" : 1}, no_cursor_timeout = True)
    for a in nodes:
        insert_node(session, a["id"])


def insert_node(session: Session, nodeid: str):
    session.run("CREATE (node: RoadNode { nodeid: $nodeid })",
                nodeid = nodeid)


def delete_nodes(driver: neo4j.Driver):
    session = driver.session()
    session.run("MATCH (n)"
                "DETACH delete n")

    
def delete_relationships(driver: neo4j.Driver):
    session = driver.session()
    session.run("MATCH ()-[r:can_drive_to]-()"
                "DELETE r")

    
def create_relationships(driver: Driver, mongoclient: MongoClient):
    with mongoclient.start_session() as mongo_session:
        db = mongoclient["roaddata"]
        nodecollection = db["nodes"]
        linkcollection = db["links"]
        neo_session = driver.session()
        
        for a in nodecollection.find({}, no_cursor_timeout = True):
            fromnode_id = a["id"]
            for b in a["links"]:
                link = linkcollection.find_one({ "reference" : b })
                if a["id"] == link["startnode"]:
                    fromnodetype = "startnode"
                    destnodetype = "endnode"
                else:
                    fromnodetype = "endnode"
                    destnodetype = "startnode"
                    
                
                # startnode = link["startnode"]
                # endnode = link["endnode"]
                reference: str = link["reference"]
                length: int = link["length"]

                lanes: list[int] = link["lanes"]
                if len(link["speedlimits"]) == 0:
                    speedlimit = 50
                else:
                    speedlimit = round(sum(map(lambda x: x["value"], link["speedlimits"])) / len(link["speedlimits"]))

                coordinates = list(itertools.chain(*link["geometry"]["coordinates"]))
                lats = coordinates[1::2]
                lngs = coordinates[0::2]
                
                if can_drive(lanes, fromnodetype):
                    create_relationship(neo_session, link[fromnodetype], link[destnodetype], reference, length, speedlimit, lats, lngs)
                # if can_drive(lanes, "endnode"):
                    # create_relationship(neo_session, endnode, startnode, reference, length, speedlimit, lats, lngs)

                    
def create_relationship(session: Session, fromnode_id: str, destnode_id: str, reference: str, length: int, speedlimit: int, lats, lngs):
    session.run("MATCH (from_node: RoadNode { nodeid: $fromnode_id })"
                "MATCH (dest_node: RoadNode { nodeid: $destnode_id })"
                "CREATE (from_node)-[rel:can_drive_to { reference: $reference, length : $length, speedlimit : $speedlimit, lats : $lats, lngs : $lngs }]->(dest_node)",
                fromnode_id = fromnode_id,
                destnode_id = destnode_id,
                reference = reference,
                length = length,
                speedlimit = speedlimit,
                lats = lats,
                lngs = lngs)


def can_drive(lanes, nodetype: str) -> bool:
    if len(lanes) == 0:
        return True
    if nodetype == "startnode":
        if len(list(filter(lambda x: x % 2 != 0, lanes))) != 0:
            return True
    if nodetype == "endnode":
        if len(list(filter(lambda x: x % 2 == 0, lanes))) != 0:
            return True
    return False


def create_node_indexes(driver: Driver):
    session = driver.session()
    session.run("CREATE INDEX ON :RoadNode(nodeid)")

    
def main():
    mongoclient = MongoClient("localhost",
                              27017,
                              # username = "skrivebruker",
                              # password = "skrivepassord",
                              authSource  = "roaddata",
                              ssl = False)

    neodriver = GraphDatabase.driver("bolt://localhost:7687",
                                     auth = ("neo4j", "neopassword"))
    mongodb = mongoclient["roaddata"]
    nodecollection: collection.Collection = mongodb["nodes"]
    linkcollection = mongoclient["links"]

    delete_nodes(neodriver)
    insert_nodes(neodriver, nodecollection)

    try:
        create_node_indexes(neodriver)
    except ClientError:
        pass
    
    delete_relationships(neodriver)
    create_relationships(neodriver, mongoclient)    
main()
