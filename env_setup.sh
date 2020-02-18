#!/bin/bash

if [[ $(id -u) -ne 0 ]]; then
    echo "Script must be run as root. Please re-run with 'sudo $0'"
    exit 1
fi

# logname doesn't work on WSL
REALUSER=$(logname 2>/dev/null)
if [[ "$REALUSER" == "" || "$REALUSER" == "root" ]]; then
    if [[ "$1" == "" ]]; then
        echo "You are likely running this script from within WSL."
        echo "If so you must specify the your username as the first argument (sudo $0 <username>)"
        exit 1
    fi
    REALUSER="$1"
fi

if [[ ! -f /etc/os-release ]]; then
    echo "Unable to determine OS-version, quitting"
    exit 1
fi

# Exit on errors
set -e

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
        apt-get install -y postgresql postgis ffmpeg pkg-config npm protobuf-compiler libprotobuf-dev nodejs-dev node-gyp libssl1.0-dev
        service postgresql start
        ;;
    *)
        echo "Unknown OS '$ID'"
        exit 1
        ;;
esac


if [[ $(sudo -u postgres psql postgres -tAc "SELECT 1 FROM pg_roles WHERE rolname='postgres'" 2>/dev/null) -ne 1 ]]; then
    echo "Creating PostgreSQL user for $REALUSER"
    sudo -u postgres createuser -drs $REALUSER
fi

if [[ $(sudo -u postgres psql postgres -tAc "SELECT 1 FROM pg_database WHERE datname='backlog'" 2>/dev/null) -ne 1 ]]; then
    echo "Creating database 'backlog'"
    sudo -u $REALUSER createdb backlog
    cargo run init-db
fi

if [ ! -L .git/hooks/pre-commit ]; then
    echo "Installing pre-commit hook"
    sudo -u $REALUSER ln -s ../../pre-commit .git/hooks/pre-commit
fi

if [ ! -d node_modules/ ]; then
    echo "Installing JavaScript dependencies"
    npm install
fi

echo "Done"
