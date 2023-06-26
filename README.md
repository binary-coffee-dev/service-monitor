# BinaryCoffee service monitor

This service is meant to mointoring the BinaryCoffee services and notify on telegram possible errors.

## List of monitored services

- [x] Application vitality
- [x] Application certificate
- [x] Application frontend is working

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

## todo

- [ ] Add help, and support for application arguments.
- [ ] When bot is in pause, it should be alerting the user every X time about it.

