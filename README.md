# BinaryCoffee service monitor

This service monitors BinaryCoffee website and promptly notified potential errors via Telegram for efficient troubleshooting.
The service can be re-utilized just by updating the configuration file to your needs.

## List of monitored services

- [x] Application vitality
- [x] Application certificate
- [x] Application frontend is working
- [x] Endpoint to rise directly notifications

## Project dev

### Configurations

The application configurations is a file with the following structure:

```
{
  // notification api configuration
  "enable_api": true,
  "host": "127.0.0.1",
  "port": 6565,
  "api_token": "example_token",

  // telegram bot integration
  "telegram_bot_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11",
  "groups": [149770819],

  // time interval to automatically check the monitored system
  "website_monitor_timeout": 20,
  
  // list of api endpoints to check
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
  
  // list of frontend endpoints to check
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
  
  // lise of domains to validate SSL certificate
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

### Notification API

The notification API is used to manually prompt notifications in Telegram.
This could be used to integrate your project with the monitoring service, and sent useful notifications to the Telegram chanel.

The integration is quite simple, and it can be done but filling the information in the configuration file, and then make a POST request to the endpoint `/notification`.
The body of the request should be a **json** with the following format:

```json
{
  "message": "my message"
}
```

For security reasons, the request use basic auth.
This means that you need to inject in the POST request the token in the following format:

```text
Authorization: Basic <base64_token>

Ex:
Authorization: Basic dGVzdA==
```

## toDo

- [ ] Add help, and support for application arguments.
- [ ] Add integration tests (code is not well tested)
- [ ] Allow to define the default route for the configuration file
- [ ] Before test an url, ping the domain to see if is available

