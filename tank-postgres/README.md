# tank-postgres
Postgres driver implementation for [Tank](https://crates.io/crates/tank): the Rust data layer

## Run tests
Running the tests require a instance of Postgres, you can provide the connection url using the environment variable `TANK_POSTGRES_TEST`. Otherwise a containerized docker image will be be launched using [testcontainers-modules](https://crates.io/crates/testcontainers-modules). 
1. Make sure docker is running
```ssh
systemctl status docker
```
2. Add your user to the docker group
```ssh
sudo usermod -aG docker $USER
```
You might need to logout for this to take effect
