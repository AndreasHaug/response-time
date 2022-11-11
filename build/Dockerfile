FROM python:3.10.8

WORKDIR /build_db

RUN apt update -y
RUN apt upgrade -y

COPY ./requirements.txt .
COPY ./build_db/*.py .
COPY ./build_db/*.sh .
COPY ./nvdb_client/get_data.py .

RUN rm -fr DATA

RUN pip install -r requirements.txt

CMD python3 get_data.py && \
    python3 load_raw.py \
    mongodb://database/roaddata \
    DATA/veglenkesekvenser DATA/fartsgrenser/ \
    27017 \
    roaddata raw_links raw_speedlimits && \
    python3 extract_speedlimits_from_raw.py \
    database 27017 roaddata raw_speedlimits speedlimits && \
    python load_from_raw.py \
    database 27017 roaddata \
    raw_links links speedlimits nodes points && \
    echo "Finished building database"

EXPOSE 3000
