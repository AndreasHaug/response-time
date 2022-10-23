docker run --net=host --rm -v  $(pwd)/mongodb:/data/db --memory="1g" mongo:6.0  mongod
