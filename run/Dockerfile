FROM rust:1.64

RUN apt update
RUN apt install apt-utils -y
RUN apt upgrade -y

RUN groupadd app
RUN useradd -g app app

WORKDIR /frontend
COPY ./frontend .
RUN chown -R app:app .
WORKDIR /datastructure
COPY ./datastructure .
RUN chown -R app:app .


USER app

# RUN rustup default nightly
# RUN cargo build --release

CMD rustup default nightly && \
    MONGO_DB_CONNECTION="database:27017" \
    CLOSEST_URL="http://closest:8001/" \
    cargo run --release

EXPOSE 8000
