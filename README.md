

# Watchfile
![Static Badge](https://img.shields.io/badge/version-0.1.1-blue)

Watchfile is a server that monitors a specified Markdown file for changes and notifies via WebSocket when a change is detected.

## Usage

1. Edit the `config.yaml` file to set the path of the Markdown file you want to monitor in the `watch_path` field.
2. Start the server by running the `cargo run` command.
3. Connect to the server using a WebSocket client at `ws://127.0.0.1:9000/ws`.

## Directory Structure
```
.
├── watchfile.exe  <- run
├── config.yaml
├── note.md        <- watch target
└── static
    ├── index.html    `127.0.0.1/`
    ├── test.html     `127.0.0.1/test`
    ├── :
    └── yourfile.html    `127.0.0.1/yourfile.html`
```

## Version check

`127.0.0.1/version`
```json
{
    "app_name": "this app name",
    "version": "app version",
    "repository": "URL",
}
```

## Features

* Monitors a Markdown file for changes.
* Notifies via WebSocket when a change is detected.
* The notified content is the HTML-formatted content of the changed Markdown file.

## Dependencies

* `actix-web` crate for building the server
* `markdown` crate for parsing Markdown files

## Configuration

The `config.yaml` file is used to configure the server. The following fields are available:

* `watch_path`: The path to the Markdown file to monitor.
* `server.address`: The address to bind the server to.
* `server.port`: The port to bind the server to.

## Example Configuration

```yml
watch_path: "./note.md"
server:
  address: "127.0.0.1"
  port: 9000
```

## License

This project is licensed under the MIT License.