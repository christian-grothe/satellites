#!/bin/bash

echo "starting the server"

(cd server && cargo run ) &

echo "starting watcher"

(cd watcher && cargo run) &

echo "starting frontend"
(cd frontend && npm run dev) &

wait

