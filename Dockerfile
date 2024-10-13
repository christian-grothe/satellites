# Description: Dockerfile for Rust development environment
# compiling Rust code in a Debian image to use with raspberry pi

FROM debian:latest

# Update default packages
RUN apt-get update

# Get Ubuntu packages
RUN apt-get install -y \
    build-essential \
    curl

# Update new packages
RUN apt-get update

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

ENTRYPOINT ["/bin/bash"]
