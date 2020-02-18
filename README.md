# Backlog
Personal digital photography organisation program.


## Development environment setup
The following instructions are tested on:

* Fedora Workstation 31
* Ubuntu 18.04 LTS in Windows Subsystem for Linux (WSL)

First install Rust 1.40 or later using [Rustup](https://rustup.rs/).

The following command will automatically install all other requirements.

```bash
sudo ./env_setup.sh
```


## Frontend
```bash
npm install  # Only necessary when package.json has changed or when cloning the repository
npm run watch
```
