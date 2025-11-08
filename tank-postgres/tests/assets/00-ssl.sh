#!/bin/bash
set -euo pipefail

echo '[ssl-init] Configuring Postgres SSL (copying files into PGDATA)'
cp /docker-entrypoint-initdb.d/ca.crt     "$PGDATA"/ca.crt
cp /docker-entrypoint-initdb.d/server.crt "$PGDATA"/server.crt
cp /docker-entrypoint-initdb.d/server.key "$PGDATA"/server.key
chmod 600  "$PGDATA"/server.key || true
chmod 644  "$PGDATA"/server.crt "$PGDATA"/ca.crt || true

echo "ssl=on"                     >> "$PGDATA"/postgresql.conf
echo "ssl_ca_file='ca.crt'"       >> "$PGDATA"/postgresql.conf
echo "ssl_cert_file='server.crt'" >> "$PGDATA"/postgresql.conf
echo "ssl_key_file='server.key'"  >> "$PGDATA"/postgresql.conf
echo "listen_addresses='*'"       >> "$PGDATA"/postgresql.conf

# Replace pg_hba.conf with our rules
cp /docker-entrypoint-initdb.d/pg_hba.conf "$PGDATA"/pg_hba.conf
