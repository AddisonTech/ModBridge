# ModBridge

> Modbus TCP utility written in Rust -- poll live register values, serve them as a REST API, log to CSV, or spin up a fake device for testing.

Same feature set as [ModBridge_py](https://github.com/AddisonTech/ModBridge_py) with the performance and single-binary distribution of Rust.

---

## Installation

[Rust](https://rustup.rs) required.

```sh
git clone https://github.com/AddisonTech/ModBridge
cd ModBridge
cargo install --path .
```

After install, `modbridge` is available anywhere in your shell.

---

## Commands

### `simulate` -- built-in fake Modbus server

Start a local fake device with 100 holding registers (40001-40100) whose values random-walk every second. No real hardware needed.

```sh
modbridge simulate
```

```
  ModBridge simulator on localhost:502
  Serving 100 registers (40001-40100) with random-walking values
  Press Ctrl+C to stop
```

Point any other command at `localhost` to test against it.

---

### `poll` -- live terminal display

Connects to a Modbus device and prints an updated register table on each interval.

```sh
modbridge poll --host 192.168.1.10 --registers 40001-40010 --interval 1
```

| Flag | Default | Description |
|---|---|---|
| `--host` | required | Modbus device IP |
| `--port` | `502` | Modbus TCP port |
| `--registers` | required | Register range, e.g. `40001-40010` |
| `--interval` | `1.0` | Poll interval in seconds |
| `--unit-id` | `1` | Modbus slave/unit ID |

---

### `serve` -- REST API

Polls a Modbus device in the background and exposes current register values over HTTP.

```sh
modbridge serve --host 192.168.1.10 --registers 40001-40100 --port 3000
```

| Flag | Default | Description |
|---|---|---|
| `--host` | required | Modbus device IP |
| `--port` | `3000` | HTTP API port |
| `--modbus-port` | `502` | Modbus TCP port |
| `--registers` | required | Register range |
| `--interval` | `1.0` | Background poll interval |
| `--unit-id` | `1` | Modbus slave/unit ID |

**Endpoints:**

```
GET /registers              returns all polled registers as JSON
GET /registers/{address}    returns a single register by Modbus address
```

**Example response:**

```json
{
  "registers": {
    "40001": 2341,
    "40002": 1987,
    "40003": 3102
  },
  "last_updated": "2026-05-18T14:22:01Z",
  "error": null
}
```

---

### `log` -- CSV logging

Writes a timestamped row of register values to a CSV file on every poll cycle until Ctrl+C.

```sh
modbridge log --host 192.168.1.10 --registers 40001-40010 --output data.csv
```

| Flag | Default | Description |
|---|---|---|
| `--host` | required | Modbus device IP |
| `--port` | `502` | Modbus TCP port |
| `--registers` | required | Register range |
| `--output` | required | CSV file path (appends if exists) |
| `--interval` | `1.0` | Poll interval in seconds |
| `--unit-id` | `1` | Modbus slave/unit ID |

---

## Smith_Agentic Integration

When `serve` mode is running, any Smith_Agentic agent can poll live field data directly from the REST API.

**Start serve mode against the built-in simulator:**

```sh
# Terminal 1 -- start the fake device
modbridge simulate

# Terminal 2 -- expose it over HTTP
modbridge serve --host localhost --registers 40001-40100 --port 3000
```

**Read all registers with curl:**

```sh
curl http://localhost:3000/registers
```

**Read a single register:**

```sh
curl http://localhost:3000/registers/40015
```

**Sample Smith_Agentic goal:**

> "Fetch the current register values from http://localhost:3000/registers. Flag any register whose value exceeds 4500 as a potential overrange condition. Generate a brief summary report listing each flagged register, its value, and a recommended action."

---

## Project Structure

```
ModBridge/
  src/
    main.rs          # CLI entry point
    client.rs        # Modbus TCP client
    simulator.rs     # Raw TCP Modbus server
    logger.rs        # CSV writing
    api.rs           # axum REST API
    display.rs       # Colored terminal output
    commands/
      poll.rs
      serve.rs
      log.rs
      simulate.rs
```

---

## Related

- [ModBridge_py](https://github.com/AddisonTech/ModBridge_py) -- Python implementation with identical feature set

---

## License

MIT -- see [LICENSE](LICENSE)
