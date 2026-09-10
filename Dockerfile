FROM postgres

COPY ./db/init-scripts/init.sql /docker-entrypoint-initdb.d/

RUN chmod 644 /docker-entrypoint-initdb.d/*.sql
