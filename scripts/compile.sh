#!/bin/bash

while getopts u:a:f: flag
do
    case "${flag}" in
        u) username=${OPTARG};;
        a) age=${OPTARG};;
        f) fullname=${OPTARG};;
    esac
done

echo "Username: $username";
echo "Age: $age";
echo "Full Name: $fullname";

echo "starting docker container"
docker run -dit --name cross-compile --platform linux/aarch64 cross-compile

echo "moving server to container and compiling"
docker cp server cross-compile:/tmp 
docker exec cross-compile /bin/bash -c "source /etc/profile && cd /home/server && cargo build --release"

echo "copying binary to host"
docker cp cross-compile:/home/server/target/release/server ./bin/  

# echo "copying binary to target"
# scp ./bin/server christian@satellites:/home/christian/satellites  # Specify the correct binary to copy
#

echo "removing container"
docker container stop cross-compile
docker container rm cross-compile

echo "building and copy frotend"
cd frontnend && npm run build && scp dist christian@satellites:/home/christian/satellites
