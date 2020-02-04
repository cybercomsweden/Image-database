# Backlog
Personal digital photography organisation program.

## Installation
### Fedora 31:
```
sudo dnf install postgresql-server postgis ffmpeg pkg-config openssl-devel npm protobuf-compiler protobuf-devel
sudo -u postgres /usr/bin/postgresql-setup --initdb
sudo systemctl start postgresql
sudo -u postgres createuser -drs $USER
```


### Ubuntu 18:
```
sudo apt install postgresql postgis ffmpeg pkg-config libssl-dev npm protobuf-compiler libprotobuf-dev
service postgresql start
sudo -u postgres createuser -drs $USER
```

## Using:
To create and run the data base:
```
createdb backlog
cargo run init-db
cargo run
```

To clean up the data base:
```
dropdb backlog
```

This will create a database superuser (-s) which can create new databases (-d) and new roles (-r) and a database named backlog.

## Frontend
```bash
npm install  # Only necessary when package.json has changed or when cloning the repository
npm run parcel watch src/index.jsx
```
