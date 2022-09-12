docker run \
       -p 7474:7474 -p 7687:7687 \
       --rm \
       -v $(pwd)/neo4j/data:/data \
       -v $(pwd)/neo4j/logs:/logs \
       -v $(pwd)/neo4j/import:/var/lib/neo4j/import \
       -v $(pwd)/neo4j/plugins:/plugins \
       --env NEO4J_AUTH=neo4j/neopassword \
       neo4j:4.4.0
