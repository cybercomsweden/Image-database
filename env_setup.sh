#!/bin/bash


if [[ $(id -u) -ne 0 || $(logname) == "root" ]]; then
    echo "Script must not be run as root using sudo"
    exit 1
fi

if [[ ! -f /etc/os-release ]]; then
    echo "Unable to determine OS-version, quitting"
    exit 1
fi

# Exit on errors
set -e

USER=$(logname)

source /etc/os-release

case "$ID" in
    fedora)
        dnf install -y postgresql-server postgis ffmpeg pkg-config openssl-devel npm protobuf-compiler protobuf-devel

        pgdata="/var/lib/pgsql/data"
        if [[ ! -d $pgdata || ! "$(ls -A $pgdata)" ]]; then
            echo "Creating a default PostgreSQL cluster"
            sudo -u postgres /usr/bin/postgresql-setup --initdb
        fi

        systemctl start postgresql
        ;;
    ubuntu)
        apt-get update
        apt-get install -y postgresql postgis ffmpeg pkg-config libssl-dev npm protobuf-compiler libprotobuf-dev
        service postgresql start
        ;;
    *)
        echo "Unknown OS '$ID'"
        exit 1
        ;;
esac


if [[ $(sudo -u postgres psql postgres -tAc "SELECT 1 FROM pg_roles WHERE rolname='postgres'" 2>/dev/null) -ne 1 ]]; then
    echo "Creating PostgreSQL user for $USER"
    sudo -u postgres createuser -drs $USER
fi

if [[ $(sudo -u postgres psql postgres -tAc "SELECT 1 FROM pg_database WHERE datname='backlog'" 2>/dev/null) -ne 1 ]]; then
    echo "Creating database 'backlog'"
    sudo -u $USER createdb backlog
    cargo run init-db
fi

echo "Done"
