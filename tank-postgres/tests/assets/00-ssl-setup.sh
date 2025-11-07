#!/bin/bash
set -e
chown postgres:postgres /var/lib/postgresql/data/server.key /var/lib/postgresql/data/server.crt
chmod 600 /var/lib/postgresql/data/server.key
