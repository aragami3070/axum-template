FROM postgres

COPY ./db/init-scripts/ /docker-entrypoint-initdb.d/

RUN chmod 644 /docker-entrypoint-initdb.d/*.sql
