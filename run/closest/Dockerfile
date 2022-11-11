FROM python:3.10.8

RUN apt update
RUN apt install apt-utils -y
RUN apt upgrade -y

RUN groupadd app
RUN useradd -g app app
USER app

WORKDIR /closest

COPY ./closest.py .
COPY ./requirements.txt .

USER root
RUN pip install -r requirements.txt

USER app

CMD MONGO_DB_CONNECTION=database python3 closest.py
EXPOSE 8001
