# Binary monitor

This service monitors the BinaryCoffee website and promptly notifies potential errors via Telegram for efficient
troubleshooting. The service can be re-utilized just by updating the configuration file to your needs.

## List of features

- [x] Application vitality: Allows for monitoring the vitality of the application frontend and api endpoints.
- [x] Application certificate: Monitors the SSL certificates of the domains defined in the configuration file.
- [x] Application frontend vitality: Monitors the frontend endpoints defined in the configuration file.
- [x] Endpoint to rise directly notifications: Provides an endpoint to directly send notifications to the Telegram
  channel.

## Configurations

The application configuration is a file with the following structure:

```
{
  // Notification api configuration
  // enable or disable the notification api
  "enable_api": true,
  // host where the api will be exposed
  "api_host": "0.0.0.0",
  // port where the api will be exposed
  "api_port": 6565,
  // token to access the api (to use the basic, auth you need to encode it in base64)
  "api_token": "example_token",

  // telegram bot integration
  // enable or disable telegram integration (if disabled, no commands will be monitored from telegram)
  "enable_telegram_bot_commands": true,
  // time interval to retrieve the commands from Telegram
  "retrieve_commands_interval": 2,
  // telegram bot token
  "telegram_bot_token": "",
  // group chat IDs to send the notifications
  "groups": [149770819],

  // enable or disable the monitoring service
  "enable_monitoring_service": true,
  // time interval to automatically check the monitored system
  "website_monitoring_interval": 20,
  // time interval to remind about the paused monitoring service
  "pause_reminder_interval": 86400,
  
  // list of api endpoints to check
  "api_tests": [
    {
      // type of request (GET, POST)
      "type": "POST",
      // endpoint url
      "url": "https://google.com",
      // body of the request
      "body": "{}",
      // content type of the request
      "content_type": "application/json"
    },
    {
      "type": "GET",
      "url": "https://google.com"
    }
  ],
  
  // list of frontend endpoints to check
  "frontend_tests": [
    {
      // type of request (GET)
      "type": "GET",
      // frontend url
      "url": "https://google.com"
    }
  ],
  
  // list of domains to validate SSL certificate
  "ssl_tests": [
    {
      // domain to check certificate
      "url": "google.com"
    }
  ]
}
```

> Note: the *config.json* file should be in the same folder as the application. For reference, check the
> config.initial.json file.

> Note: all time intervals are in seconds.

## Run project

All methods to run the project need to have the *config.json* file in the root directory of the project.

### Run with docker

To run the project with docker, execute the following command:

```shell
docker run -d -p 6565:6565 -v ./config.json:/config.json --name binary-monitor ggjnez92/binary-monitor:2.1.0
```

### Run with docker-compose

First, use the following docker-compose.yml file:

```yaml
services:
  binary-monitor:
    image: ggjnez92/binary-monitor:2.1.0
    container_name: binary-monitor
    restart: always
    ports:
      - 6566:6566/tcp
    volumes:
      - ./config.json:/config.json
```

Then run the following command: `docker-compose up --build -d`
