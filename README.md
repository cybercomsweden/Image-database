# Backlog
Personal digital photography organisation program.

## Installation
### Fedora 31:
```
sudo dnf install postgresql-server postgis ffmpeg pkg-config openssl-devel
sudo -u postgres /usr/bin/postgresql-setup --initdb
sudo systemctl start postgresql
sudo -u postgres createuser -drs $USER
```


### Ubuntu 18:
```
sudo apt install postgresql postgis ffmpeg pkg-config libssl-dev
service postgresql start
sudo -u postgres createuser -drs $USER
```

## Using:
To create and run the data base:
```
createdb backlog
cargo run init-db
cargo run --release
```

To clean up the data base:
```
"dropdb backlog"
```

This will create a database superuser (-s) which can create new databases (-d) and new roles (-r) and a database named backlog.

## Frontend
```
npm install
```
To generate protobuf files:(path/to/protoc/protoc.exe --js_out=import_style=commonjs,binary:. src/entity.proto

To compile javascript code:
sudo npm install -g yarn

sudo npm install -g parcel-bundler

parcel watch src/index.jsx
