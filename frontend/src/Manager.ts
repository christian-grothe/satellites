import GranularSynth from "./GranularSynth";
import { parseOSCMessage } from "./oscParser";
import { Stars } from "./Stars";

export class Manager {
  granularSynth: GranularSynth;
  stars: Stars;
  ws: WebSocket | undefined;
  ctx: AudioContext | undefined;
  analyserNode: AnalyserNode | undefined;
  mstrVol: GainNode | undefined;
  offsets: Array<number>;
  offsetAvergae: number;
  _recordingList: string[];
  audioBuffers: AudioBuffer[];
  apiUrl: string;
  webSocketProtocol: string;
  httpProtocol: string;

  constructor() {
    this.granularSynth = new GranularSynth();
    this.offsets = [];
    this.offsetAvergae = 0;
    this._recordingList = [];
    this.audioBuffers = [];
    this.stars = new Stars(100);
    this.apiUrl = import.meta.env.VITE_SERVER_URL;
    this.webSocketProtocol = import.meta.env.VITE_WEBSOCKET_PROTOCOL;
    this.httpProtocol = import.meta.env.VITE_HTTP_PROTOCOL;
  }

  async init(ctx: AudioContext) {
    this.ctx = ctx;

    this.mstrVol = ctx.createGain();
    this.mstrVol.gain.value = 1;
    this.analyserNode = ctx.createAnalyser();
    this.mstrVol.connect(this.analyserNode);
    this.analyserNode.connect(ctx.destination);
    this.analyserNode.fftSize = 256;

    await this.granularSynth.init(ctx);

    this.stars.setAnalyser(this.analyserNode);

    const ws = new WebSocket(
      `${import.meta.env.VITE_WEBSOCKET_PROTOCOL}://${import.meta.env.VITE_SERVER_URL}:${import.meta.env.VITE_SOCKET_PORT}/ws`,
    );

    ws.onopen = () => {
      this._requestSync();
      // Periodically resync so slow clock drift is corrected during a
      // performance (the rolling average keeps the last 20 offsets).
      setInterval(() => {
        this._requestSync();
      }, 5000);
    };

    ws.onmessage = this._handleMessages.bind(this);

    this.ws = ws;
  }

  async _handleMessages(event: MessageEvent<Blob>) {
    const arrayBuf = await event.data.arrayBuffer();

    const { address, args } = parseOSCMessage(arrayBuf);

    switch (address) {
      case "/sampler/play":
      case "/sampler/play/rand":
      case "/sampler/play/next":
        this.granularSynth.setAndPlay(
          this.audioBuffers,
          args,
          this.offsetAvergae,
        );
        break;
      case "/recordings":
        this.recordingList = args as string[];
        break;
      case "/test":
        console.log(args);
        break;
      case "/sync": {
        const t3 = performance.timeOrigin + performance.now();
        const t1 = args[1] as number;
        const t2 = args[3] as number;
        this._handleOffset(t1, t2, t3);
        break;
      }
      default:
        console.error("unknown OSC address", address);
        break;
    }
  }

  set recordingList(filenames: string[]) {
    console.log(filenames);
    this._recordingList = filenames;
    setTimeout(() => {
      this.getRecordings();
    }, 500);
  }

  async getRecordings() {
    if (!this.ctx) return;
    this.audioBuffers = [];
    try {
      for (const filename of this._recordingList) {
        if (filename === "test.wav") continue;
        const response = await fetch(
          `${import.meta.env.VITE_HTTP_PROTOCOL}://${import.meta.env.VITE_SERVER_URL}:${import.meta.env.VITE_STATIC_PORT}/${filename}`,
        );
        const arrayBuffer = await response.arrayBuffer();
        const newBuffer = await this.ctx.decodeAudioData(arrayBuffer);
        this.audioBuffers.push(newBuffer);
      }
    } catch (e) {
      console.log(e);
    }
  }

  _handleOffset(t1: number, t2: number, t3: number) {
    const rtt = t3 - t1;
    const owd = rtt / 2;
    // NTP offset (how far the server clock is ahead of ours):
    //   offset = t2 - (t1 + t3) / 2 = t2 - t1 - owd
    const offset = t2 - t1 - owd;
    this.offsets.push(offset);
    if (this.offsets.length > 20) {
      this.offsets.shift();
    }
    this.offsetAvergae =
      this.offsets.reduce((a, b) => a + b) / this.offsets.length;
  }

  _requestSync() {
    if (!this.ws) return;
    const now = performance.timeOrigin + performance.now();
    const msg = { message_type: "sync", data: `${now}` };
    this.ws.send(JSON.stringify(msg));
  }
}
