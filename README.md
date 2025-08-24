# Http Request Fanout Forwarder

A lightweight **HTTP fanout proxy** written in Rust with Actix-Web.  
It receives HTTP requests on a single endpoint and forwards them asynchronously to multiple dynamically configured targets.  
The service **always responds `200 OK` immediately** and logs any failed requests.  

---

## Features

- 🚀 High-performance fanout using **Actix-Web + Reqwest**  
- 🔄 **Dynamic target configuration** (add/remove/list at runtime via API)  
- 📦 Forward requests to **multiple targets** in parallel  
- ✅ Immediate `200 OK` response (non-blocking fanout)  
- 📝 Logs failures (network errors or non-2xx responses)  

---

## Installation

```bash
# Clone this repo
# Build
cargo build --release

# Run
cargo run --release
```

The service will start on **port 8080**.  

---

## API Usage

### 1. Add a target
```bash
POST /add_target/{id}
Content-Type: application/json

{
  "url": "http://localhost:9000/endpoint1"
}
```

➡ Adds a new forwarding target for the given `{id}` group.  

---

### 2. Remove a target
```bash
POST /remove_target/{id}
Content-Type: application/json

{
  "url": "http://localhost:9000/endpoint1"
}
```

➡ Removes the given URL from the `{id}` group.  

---

### 3. List targets
```bash
GET /list_targets/{id}
```

Response:
```json
{
  "id": "myid",
  "targets": [
    "http://localhost:9000/endpoint1",
    "http://localhost:9001/endpoint2"
  ]
}
```

---

### 4. Fanout request
```bash
POST /fanout/{id}/{tail...}
```

- Forwards the request to **all configured URLs** for `{id}`.  
- `{tail...}` (optional) is appended to the target URLs.  
- Headers and body are preserved.  
- Always returns immediately:

```json
{
  "status": "queued",
  "id": "myid"
}
```

Example:
```bash
curl -X POST http://localhost:8080/fanout/myid/testpath \
     -H "Content-Type: application/json" \
     -d '{"hello":"world"}'
```

This will forward the request to all configured URLs under `myid` with `/testpath` appended.  

---

## Example Flow

```bash
# Add targets
curl -X POST http://localhost:8080/add_target/myid \
     -H "Content-Type: application/json" \
     -d '{"url":"http://localhost:9000/endpoint1"}'

curl -X POST http://localhost:8080/add_target/myid \
     -H "Content-Type: application/json" \
     -d '{"url":"http://localhost:9001/endpoint2"}'

# List targets
curl http://localhost:8080/list_targets/myid

# Fanout
curl -X POST http://localhost:8080/fanout/myid/test \
     -H "Content-Type: application/json" \
     -d '{"ping":"pong"}'
```

---

## Logging

- Successful fanouts are **silent**.  
- Failures are logged to `stdout`, e.g.:

```
Fanout to http://localhost:9000/endpoint1 failed with status 500
Fanout to http://localhost:9001/endpoint2 error: connection refused
```

---

## Dependencies

- [actix-web](https://crates.io/crates/actix-web) – Web framework  
- [reqwest](https://crates.io/crates/reqwest) – HTTP client  
- [tokio](https://crates.io/crates/tokio) – Async runtime  
- [serde](https://crates.io/crates/serde) + [serde_json](https://crates.io/crates/serde_json) – JSON  

---

## License

MIT  
