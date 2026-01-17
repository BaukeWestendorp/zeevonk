# Zeevonk

A modular lighting control system for modern DMX-based lighting setups.

> ⚠️ **Warning**
>
> Zeevonk is currently in early development. APIs, features, and behavior may change frequently and without notice.
> It is **not yet recommended for production use**.

## What is Zeevonk?

Zeevonk is a modular system for controlling lighting fixtures. It consists of a server and two types of clients:

- **Server**: Manages clients, processes triggers and attribute updates, and sends DMX data.
- **Processor Client**: Calculates and sends fixture attribute values (like color, position, intensity) to the server.
- **Controller Client**: Sends triggers (such as button presses or fader moves) to the server.

## Run the Zeevonk server

You can install zeevonk as a binary (called `zv`) using the following command:

```sh
cargo install --path crates/cli
```

## Use as a library

### Features

The crate has three main features you can enable:
`server`: Start and configure the server from your own code instead of the CLI.
`client-processor`: Use the processor client.
`client-controller`: Use the controller client.

### Example: Starting the Server

```rust
use zeevonk::server::Server;
use zeevonk::project::definition::ProjectDefinition;

// Create a project definition.
let project_def = ProjectDefinition::default();

// Create and start the server.
let server = Server::new(project_def).unwrap();
server.start();
```

### Example: Processor Client

```rust
use zeevonk::client::processor::Client;
use zeevonk::value::AttributeValues;

#[tokio::main]
async fn main() -> zeevonk::client::Result<()> {
    let mut client = Client::new();
    client.connect("ws://localhost:9001").await?;
    let mut values = AttributeValues::new();
    // Set attribute values for your fixtures here...
    client.update_attributes(values, false).await?;
    Ok(())
}
```

### Example: Controller Client

```rust
use zeevonk::client::controller::Client;

#[tokio::main]
async fn main() -> zeevonk::client::Result<()> {
    let mut client = Client::new();
    client.connect("ws://localhost:7334").await?;
    client.send_trigger("button_1_pressed").await?;
    Ok(())
}
```

For more details, see the documentation for each module in the crate.
