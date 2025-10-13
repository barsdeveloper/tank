# tank-postgres
Postgres driver implementation for [Tank](https://crates.io/crates/tank): the Rust data layer

## Running Tests
Running the tests requires an instance of Postgre. You can provide the connection URL using the environment variable `TANK_POSTGRES_TEST`. If this variable is not set, a containerized Postgres instance will automatically be launched using [testcontainers-modules](https://crates.io/crates/testcontainers-modules). 
1. Make sure docker is running
```sh
systemctl status docker
```
2. Add your user to the Docker group (so you can run Docker without sudo):
```sh
sudo usermod -aG docker $USER
```
You might need to log out and back in for this change to take effect.

> [!CAUTION]
> When running tests using the Postgres instance from testcontainers, never stop or kill the test abruptly (e.g., while at a breakpoint), as the container may not be cleaned up properly. Repeatedly starting and killing tests can leave multiple Postgres containers running, quickly consuming system resources..
> 
> To inspect running containers
> ```sh
> docker ps
> ```
> To stop a specific container:
> ```sh
> docker kill <container_name>
> ```
