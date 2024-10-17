# Satellites
> A multichannel performance where the mobilephones in the audience become speakers

In this project I use Super Collider to record audio samples and to send OSC (Open Sound Protocol) messages to clients. The clients can load the audio files and when receiving a OSC message they gonna play the sample based on the data in the OSC message. With that I can controll and sequence the mobile phones. 


### server

The server listen on one port for udp messages, which are osc messages coming from super collider. And on another port handles websocket connections that forwards the osc messages to the connected clients. The server will run on a raspberry pi connected to a local wifi network. 


### watcher

The watcher simply watches the recordings folder for new recordings. If a new recording is created it will send it via ssh to the raspberry pi. So that it can forward the recroding to the clients.

### superCollider

A simple superCollider patch that sends osc messages to the server via udp. It can send messages to the next client in order to play a sequence, to a random client or broadcast it to all clients.

### frontend

A very simple frontend that is beeing serverd by the raspbery pi via nginx. The app connects to the websocket server and receives the osc messages. It runs some synths and sampler instruments made with the web audio api that can be targeted via the osc message. 


### todos
- [ ] handle ping / pong events
- [ ] work on the clients synths and sampler
- [ ] let clients interact with eachother

