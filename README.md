# BinaryCoffee service monitor

This service is meant to mointoring the BinaryCoffee services and notify on telegram possible errors.

## List of monitored services

- [x] Application vitality
- [x] Application certificate
- [x] Application frontend is working
- [ ] Endpoint to rise directly notifications

## Project dev

### Configurations

The application configurations is a file with the following structure:

```
{
  "telegram_bot_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11",
  "groups": [149770819],
  "website_monitor_timeout": 20,
  "api_tests": [
    {
      "type": "POST",
      "url": "https://api.binarycoffee.dev/graphql",
      "body": "{}",
      "content_type": "application/json"
    },
    {
      "type": "GET",
      "url": "https://api.binarycoffee.dev/api/sitemap"
    }
  ],
  "frontend_tests": [
    {
      "type": "GET",
      "url": "https://binarycoffee.dev"
    },
    {
      "type": "GET",
      "url": "https://binarycoffee.dev/post/bienvenidos-al-blog-binary-coffeermdcl"
    },
    {
      "type": "GET",
      "url": "https://binarycoffee.dev/users/guille"
    }
  ],
  "ssl_tests": [
    {
      "url": "binarycoffee.dev"
    },
    {
      "url": "api.binarycoffee.dev"
    }
  ]
}
```

> Note: the *config.json* file should be in the same folder that the application.

### Build/Start project

```
// build project
cargo build

// run project
cargo run
```

### Start with docker

To start the project with docker run the following command.

```
docker-compose up --build -d
```

> Note: before execute the previous command, create the `config.json` in the root directory of the project.

## toDo

- [ ] Add help, and support for application arguments.
- [ ] Fix spaghetti code (hideous code)
- [ ] Add test coverage (code is not tested)
- [ ] Add argument to the script, so is possible to disable functionalities when is executed
- [ ] Add documentation when the bot gets the start command from Telegram
  - [ ] Add the documentation as a configurable parameter
- [ ] Setting up an endpoint to be able to publish alerts by demand from the API

