#!/bin/bash

frontend=0
backend=0

user="christian"
address="satellites.local"
path="/home/christian/satellites"

while getopts "fb" opt; do
  case $opt in
    f)
      frontend=1
      ;;
    b)
      backend=1
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      ;;
  esac
done

if [ $frontend -eq 0 ] && [ $backend -eq 0 ]; then
  frontend=1
  backend=1
fi


if [ $backend -eq 1 ]; then
  echo "building backend"

  ssh ${user}@${address} 'sudo systemctl stop satellites.service'

  cd server
  echo "compiling for aarch64"
  cross build --target aarch64-unknown-linux-gnu --release

  echo "copying binary to pi \n\n"

  cd target/aarch64-unknown-linux-gnu/release
  mv server server_aarch64
  scp server_aarch64 \
  ${user}@${address}:${path}/

  ssh ${user}@${address} 'sudo systemctl restart satellites.service'

  cd ../../../../
fi

if [ $frontend -eq 1 ]; then
  echo "building frontend"

  cd frontend

  npm run build

  echo "copying frontend to pi"
  scp -r dist/ ${user}@${address}:${path}/

  ssh christian@satellites.local 'cd /home/christian/satellites/ && ./deploy.sh'
fi
