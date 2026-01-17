# Zeevonk

A modular lighting control system that serves as a hub for lighting communication, processing, and interaction.

> ⚠️ **Warning**
>
> Zeevonk is currently in early development. APIs, features, and behavior may change frequently and without notice.
> It is **not yet recommended for production use**.

## The Server

**Note:** The `server` feature must be enabled to start and manage a server from your own code. If you prefer a ready-made program instead of embedding a server, use the standalone zeevonk command-line tool.

The Zeevonk server is a hub for managing clients. Its essential responsibilities include:

- Receiving triggers from controller clients and routing them to the correct processor clients.
- Receiving attribute updates from processor clients and converting them to DMX output.
- Sending DMX output over various protocols, such as sACN or Entecc Open DMX.

## The Processor Client

**Note:** The `client-processor` feature must be enabled to use a processor client in your code.

A processor client is responsible for generating high-level [GDTF](https://gdtf.eu) attribute values for specific fixtures and sending them to the server.

Typical responsibilities of a processor client include:

- Subscribing to triggers.
- Mapping triggers to fixture/attribute targets and resolving which attributes should change.
- Calculating or interpolating attribute values (effects, fades, curves, color mixing, etc.).
- Sending attribute updates to the server for DMX output.
- Maintaining local state and managing transitions to ensure updates are smooth and deterministic.

## The Controller Client

**Note:** The `client-controller` feature must be enabled to use a controller client in your code.

A controller client is the origin of triggers.

Typical responsibilities of a controller client include:

- Sending triggers (MIDI, OSC, button presses, fader changes, cue selections, etc.) to the server.

## Examples

FIXME: Add examples.
