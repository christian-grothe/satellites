# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

**Satellites** is a multichannel sound-art performance system: audience members' phones become
a distributed speaker array. SuperCollider records audio samples and sequences playback by
sending OSC messages; a Rust server fans those messages out over WebSockets to browser clients
(the phones), which play the samples through the Web Audio API. The whole stack runs on a
Raspberry Pi on a local Wi-Fi network during a performance.

## Components

The repo is four loosely-coupled programs plus deployment glue:

- **`server/`** (Rust, tokio) — the hub. Binds **UDP :8081** for incoming OSC from SuperCollider
  and **TCP :8080** for client WebSockets. Forwards OSC to clients and handles clock sync.
- **`frontend/`** (TypeScript + Vite, no framework) — the client that runs on each phone.
  Connects to the WebSocket, decodes OSC, drives Web Audio synths/samplers.
- **`watcher/`** (Rust) — runs on the *recording machine* (dev laptop). Watches `recordings/`
  and `scp`s new `.wav` files to the Pi. Hardcoded destination in `watcher/src/main.rs`.
- **`superCollider.scd`** — the performer's patch. Records 5s buffers, writes timestamped `.wav`
  files into `recordings/`, and sends `/sampler/play*` OSC sequences to the server over UDP.

## Data flow (the big picture)

1. SuperCollider records a sample → writes `recordings/<DDMMHHMMSS>.wav`.
2. `watcher` detects the new file → `scp` to the Pi's `recordings/`.
3. On the Pi, the `server`'s `notify` watcher sees the new file → broadcasts an updated
   `/recordings` OSC list to all connected clients.
4. Each client fetches the new `.wav`s over HTTP (from the static nginx, see below) and
   `decodeAudioData`s them into `audioBuffers`.
5. During performance, SuperCollider sends `/sampler/play`, `/sampler/play/next`, or
   `/sampler/play/rand` over UDP. The server appends a `timestamp` arg (now + 1000ms) and
   routes: `play` → broadcast, `play/next` → round-robin one client, `play/rand` → random
   client. Clients schedule playback at that timestamp, corrected by their clock offset.

### OSC over WebSocket

OSC packets are sent to clients as **binary WebSocket frames** (`Message::Binary`), not osc-js.
`frontend/src/oscParser.ts` is a **hand-written OSC decoder** — `osc-js` is a dependency but the
custom parser is what's used in `Manager._handleMessages`. Sampler play args are encoded as
flat `[key, value, key, value, ...]` pairs and re-assembled into an object in
`Sampler.setAndPlay`.

### Clock synchronization (NTP-style)

Phones need synchronized playback. On connect the client sends a JSON `{message_type: "sync",
data: "<t1>"}` text frame. The server replies with an OSC `/sync` message carrying `t1` (client
send time) and `t2` (server receive time). `Manager._handleOffset` computes
`offset = t2 - t1 + rtt/2`, keeps a rolling average of the last 20, and `Sampler` uses it to
convert the message `timestamp` into a local `AudioContext` start time. JSON is used inbound
(client→server); OSC binary is used for everything outbound.

## Running

### Local dev (Docker Compose)

```bash
docker compose -f docker-compose.dev.yml up --build
```

Brings up three services: `server` (`cargo run`, ports 8080/tcp + 8081/udp, source mounted for
rebuilds), `frontend` (`vite --host` on :5173), and `static` (nginx serving `recordings/` on
**:8083** with `autoindex` + CORS `*`). Production compose (`docker-compose.yml`) builds release
images and serves the built frontend via nginx.

### Frontend directly

```bash
cd frontend
npm install
npm run dev      # vite --host (needed so phones on the LAN can reach it)
npm run build    # tsc && vite build  — also the type-check step
```

There are no tests and no separate lint script wired into `frontend/package.json`. The **root**
`package.json` carries prettier + eslint 10 + typescript-eslint (`eslint.config.mjs`); run
`npx eslint .` / `npx prettier` from the repo root.

### Server / watcher directly

```bash
cd server && cargo run      # or cargo build --release
cd watcher && cargo run     # watches ../recordings and scp's to the Pi
```

The server expects a `./recordings` directory relative to its working directory.

## Deployment to the Raspberry Pi

`scripts/deploy.sh` is the deploy tool (host `satellites.local`, user `christian`,
`/home/christian/satellites`):

```bash
./scripts/deploy.sh        # both frontend and backend
./scripts/deploy.sh -b     # backend only
./scripts/deploy.sh -f     # frontend only
```

Backend is **cross-compiled** for the Pi: `cross build --target aarch64-unknown-linux-gnu
--release` (requires the `cross` tool + Docker), the binary is `scp`'d over, and the server runs
as a systemd unit `satellites.service`. Frontend is `npm run build` then `scp dist/` + a remote
`deploy.sh`.

## Environment-dependent endpoints

Hostnames/IPs are **hardcoded in several places** — when changing network setup, update all of:

- `frontend/src/Manager.ts` — `apiUrl` toggles `localhost:8080` (dev) vs `satellites.kryshe.com`
  (prod) via `import.meta.env.DEV`; recordings fetched from `localhost:8083`.
- `frontend/src/Sampler.ts` — `getRecordings()` fetches from `satellites.local:8080/recordings/`
  (note: this differs from `Manager.getRecordings`'s `:8083` — they are inconsistent).
- `watcher/src/main.rs` — `scp` destination IP (`192.168.1.228`).
- `superCollider.scd` — `NetAddr` target IP for OSC.
- `scripts/deploy.sh` — `address` / `path` for the Pi.

## AudioWorklets (TypeScript)

Worklet processors live in `frontend/src/worklets/*.worklet.ts` and run in the
`AudioWorkletGlobalScope`, not the DOM — so they are type-checked separately:

- **Loading**: import the processor with `?worker&url` (e.g.
  `import url from "./worklets/x.worklet.ts?worker&url"`), then `ctx.audioWorklet.addModule(url)`.
  This makes Vite bundle + transpile the TS into a standalone IIFE asset and hand back its URL —
  works in both `vite dev` and `vite build`. Do **not** put worklets in `public/` (Vite copies
  that verbatim, so `.ts` would not be transpiled). Plain `?url` also fails the build (no
  transpile); `?worker&url` is the reason it works.
- **Typing**: globals (`AudioWorkletProcessor`, `registerProcessor`, `sampleRate`,
  `currentFrame`) come from `@types/audioworklet`. It clashes with the DOM lib, so worklets have
  their own `frontend/tsconfig.worklet.json` (`lib: ["ES2020"]` — no DOM, no WebWorker; that
  package supplies the worker-scope globals itself). `src/worklets` is excluded from the main
  `tsconfig.json` and type-checked by the second `tsc -p tsconfig.worklet.json` pass in the
  `build` script. The package ships no `AudioParamDescriptor`, so declare it locally if needed.

## Conventions & gotchas

- The server uses a single `Arc<Mutex<Clients>>` shared across the UDP, WebSocket, and
  file-watch tasks. `Clients` holds both a `HashMap<SocketAddr, Tx>` and a `Vec<SocketAddr>`
  (`client_array`) to support round-robin (`/play/next`); keep both in sync when editing
  add/remove logic.
- Liveness: server pings each client every 10s and drops clients with no pong for 30s
  (`PING_INTERVAL` / `PONG_TIMEOUT` in `ws_server.rs`).
- `test.wav` is skipped client-side when loading recordings; `recordings/` is otherwise treated
  as the live sample library and is the volume mounted into the server and static containers.
- Frontend has no router/framework — entry is `main.ts` (requires a user tap to create the
  `AudioContext`, per browser autoplay rules), orchestrated by `Manager`.
