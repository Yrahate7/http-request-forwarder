#!/bin/bash
set -e

SERVER="http://localhost:8080"
TARGETS_FILE="targets.json"

echo "🚀 Starting tests..."

# 1. Clean up old file
rm -f $TARGETS_FILE
echo "{}" > $TARGETS_FILE

# 2. Add a target to id1
echo "➕ Adding http://localhost:9000/endpoint to id1"
curl -s -X POST $SERVER/add_target/id1 \
  -H "Content-Type: application/json" \
  -d '{"url":"http://localhost:9000/endpoint"}' | jq

echo "📂 File content after add:"
cat $TARGETS_FILE | jq

# 3. Add another target to id1
echo "➕ Adding http://localhost:9001/endpoint to id1"
curl -s -X POST $SERVER/add_target/id1 \
  -H "Content-Type: application/json" \
  -d '{"url":"http://localhost:9001/endpoint"}' | jq

echo "📂 File content after second add:"
cat $TARGETS_FILE | jq

# 4. List targets for id1
echo "📜 Listing targets for id1"
curl -s $SERVER/list_targets/id1 | jq

# 5. Fanout test
echo "📡 Fanout test"
curl -s -X POST $SERVER/fanout/id1/ -d '{"hello":"world"}'
echo

# 6. Remove a target
echo "➖ Removing http://localhost:9000/endpoint from id1"
curl -s -X POST $SERVER/remove_target/id1 \
  -H "Content-Type: application/json" \
  -d '{"url":"http://localhost:9000/endpoint"}' | jq

echo "📂 File content after remove:"
cat $TARGETS_FILE | jq

# 7. Final list
echo "📜 Final targets for id1"
curl -s $SERVER/list_targets/id1 | jq

echo "✅ All tests done."
