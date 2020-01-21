# Backlog
Personal digital photography organisation program.

## Installation
### Fedora 31:
```
sudo dnf install postgresql-server postgis
sudo -u postgres /usr/bin/postgresql-setup --initdb
sudo systemctl start postgresql
sudo -u postgres createuser -drs $USER
```


### Ubuntu 18:
```
sudo apt install postgresql postgis
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

