(docker compose build || docker-compose build) && \
(docker compose up --abort-on-container-exit || docker-compose up --abort-on-container-exit)
