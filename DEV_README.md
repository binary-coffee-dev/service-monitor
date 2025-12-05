# Binary Monitor Dev Guide

## Build/Start project

```
// build project
cargo build

// run project
cargo run
```

## Start with docker

To start the project with docker run the following command.

```
docker-compose up --build -d
```

> Note: before execute the previous command, create the `config.json` in the root directory of the project.

## Notification API

The notification API is used to manually prompt notifications in Telegram. This could be used to integrate your project
with the monitoring service, and sent useful notifications to the Telegram chanel.

The integration is quite simple, and it can be done but filling the information in the configuration file, and then make
a POST request to the endpoint `/notification`. The body of the request should be a **json** with the following format:

```json
{
  "message": "my message"
}
```

For security reasons, the request use basic auth. This means that you need to inject in the POST request the token in
the following format:

```text
Authorization: Basic <base64_token>

Ex:
Authorization: Basic dGVzdA==
```

## Deploy to dockerhub

```shell
git tag vX.Y.Z
git push --tags
```

## toDo

- [ ] Check https://docs.rs/warp/latest/warp/test/index.html to improve integration test
- [ ] Add help, and support for application arguments.
- [ ] Add integration tests (code is not well tested)
- [ ] Allow to define the default route for the configuration file
- [ ] Before test an url, ping the domain to see if is available

