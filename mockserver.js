const express = require("express");
const bodyParser = require("body-parser");

// helper to create a server on given port
function createServer(port) {
    const app = express();
    app.use(bodyParser.json());

    app.post("/endpoint", (req, res) => {
        console.log(`📥 Received request on port ${port}`);
        console.log("Headers:", req.headers);
        console.log("Body:", req.body);
        res.json({ status: "ok", port, received: req.body });
    });

    app.listen(port, () => {
        console.log(`✅ Mock server running on http://localhost:${port}/endpoint`);
    });
}

// start servers on 9000, 9001, 9002
[9000, 9001, 9002].forEach(createServer);
