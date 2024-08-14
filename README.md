# Rust Client-Server Cache Application

## Introduction

This project is a Rust-based client-server application designed to demonstrate a caching mechanism. The application uses
a custom protocol for communication between the client and the server. The server stores key-value pairs in a
thread-safe cache, while the client can perform operations like setting, getting, and deleting values.

## Table of Contents

- [Introduction](#introduction)
- [Installation](#installation)
- [Usage](#usage)
- [Features](#features)
- [Dependencies](#dependencies)
- [Configuration](#configuration)
- [Documentation](#documentation)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Contributors](#contributors)
- [License](#license)

## Installation

To build and run the application, ensure that you have Rust installed. Clone the repository and use Cargo to build the
project:

```bash
git clone <repository-url>
cd <repository-directory>
cargo build --release
```

## Usage

### Server Mode

To start the application in server mode:

```bash
cargo run -- --server
```

### Client Mode

To start the application in client mode:

```bash
cargo run -- --client --addr <server-address>
```

## Features

* Custom Protocol: The application uses a custom protocol for communication (proto.rs).
* Caching Mechanism: A thread-safe caching mechanism (cache.rs) that supports operations like Get, Set, Delete, and
  Keys.
* Command-Line Interface: Uses clap for parsing command-line arguments (main.rs).
* Client-Server Architecture: A simple client-server model using TCP (client.rs).

## Dependencies

`clap` for command-line parsing.
`serde` for serialization and deserialization.
`rand, regex` for additional client functionalities.
`promkit` for interactive prompts in the client.

## Configuration

The application can be configured using command-line arguments:

`--server`: Start in server mode.
`--client`: Start in client mode.
`--addr`: Specify the server address (client mode).
`--verbose`: Enable verbose output.
`--test`: Run in test mode.

## Documentation

For more detailed documentation, refer to the source code files:

- `main.rs`: Entry point and argument parsing.
- `client.rs`: Client logic.
- `cache.rs`: Caching mechanism and server logic.
- `proto.rs`: Protocol definitions.

## Examples

### Running the Server

```bash
cargo run -- --server
```

### Running the Client

```bash
cargo run -- --client --addr 127.0.0.1:8080
```

### Performing Operations

Use the client to interact with the server:

Set a value: `set <key> <value>`
Get a value: `get <key>`
Delete a value: `delete <key>`

## Troubleshooting

Connection Issues: Ensure the server is running and reachable at the specified address.
Command Parsing: Verify the command-line arguments are correct.

## Contributors

Žymantas Maumevičius - Initial development

## License

This project is licensed under the MIT License - see the LICENSE file for details.

