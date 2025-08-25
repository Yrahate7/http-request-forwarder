# Http Request Fanout Forwarder

A lightweight **HTTP fanout proxy** written in Rust with Actix-Web.  
It receives HTTP requests on a single endpoint and forwards them asynchronously and parallely to multiple dynamically configured targets.  
The service **always responds `200 OK` immediately** and logs any failed requests.  

---
