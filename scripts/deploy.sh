#!/bin/bash

cd server

echo "compiling for aarch64"
cross build --target aarch64-unknown-linux-gnu --release

echo "copying binary to pi"
scp target/aarch64-unknown-linux-gnu/release/server \
christian@satellites.local:/home/christian/satellites/server_aarch64

echo "building frontend"

cd ../frontend

npm run build

echo "copying frontend to pi"
scp -r dist/ christian@satellites.local:/home/christian/satellites/

ssh christian@satellites.local 'cd /home/christian/satellites/ && ./deploy.sh && sudo systemctl restart satellites.service'
