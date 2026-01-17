//! The controller client sends triggers (like button presses or fader moves) to the Zeevonk server.
//!
//! **Note:** The `client-controller` feature must be enabled to use a controller client in your code.
//!
//! You can write software to, for example, receive MIDI or OSC messages from a control surface,
//! and then use the controller client to send triggers to the Zeevonk server.
//! This allows you to connect your own hardware or software controls—such as MIDI controllers,
//! OSC apps, or custom UIs—to Zeevonk by translating their events into triggers
//! that the server can route to processor clients.
//!
//! ## Example
//!
//! ```rust
//! use zeevonk::client::controller::Client;
//! use zeevonk::ident::Identifier;
//!
//! #[tokio::main]
//! async fn main() -> zeevonk::client::Result<()> {
//!     let mut client = Client::new();
//!     client.connect("ws://localhost:7334").await?;
//!     client.send_trigger("button_1_pressed").await?;
//!     Ok(())
//! }
//! ```

// FIXME: Implement
