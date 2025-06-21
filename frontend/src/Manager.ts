import Sampler from "./Sampler";
//import GranularSynth from "./GranularSynth";
import { Synth } from "./Synth";
import { parseOSCMessage } from "./oscParser";
import { Stars } from "./Stars";

export class Manager {
  //granularSynth: GranularSynth;
  sampler: Sampler;
  synth: Synth;
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
    this.sampler = new Sampler();
    this.synth = new Synth();
    //this.granularSynth = new GranularSynth();
    this.offsets = [];
    this.offsetAvergae = 0;
    this._recordingList = [];
    this.audioBuffers = [];
    this.stars = new Stars(100);
    this.apiUrl = import.meta.env.DEV ? "localhost:8080" : "satellites.kryshe.com";
    this.webSocketProtocol = import.meta.env.DEV ? "ws" : "wss";
    this.httpProtocol = import.meta.env.DEV ? "http" : "https";
  }

  init(ctx: AudioContext) {
    this.ctx = ctx;

    this.mstrVol = ctx.createGain();
    this.mstrVol.gain.value = 1;
    this.analyserNode = ctx.createAnalyser();
    this.mstrVol.connect(this.analyserNode);
    this.analyserNode.connect(ctx.destination);
    this.analyserNode.fftSize = 256;

    this.sampler.init(this.ctx, this.mstrVol);
    this.synth.init(this.ctx, this.mstrVol);
    //this.granularSynth.init(ctx);

    this.stars.setAnalyser(this.analyserNode);

    const ws = new WebSocket(`${this.webSocketProtocol}://${this.apiUrl}/ws`);

    ws.onopen = () => {
      this._requestSync();
      // setInterval(() => {
      //   this._requestSync();
      // }, 100000);
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
        this.sampler.setAndPlay(args, this.offsetAvergae, this.audioBuffers);
        break;
      case "/synth/play":
        this.synth.play();
        break;
      case "/recordings":
        console.log(args);
        this.recordingList = args as string[];
        break;
      case "/test":
        console.log(args);
        break;
      case "/sync":
        const t3 = Date.now();
        const t1 = args[1] as number;
        const t2 = args[3] as number;
        this._handleOffset(t1, t2, t3);
        break;
      default:
        console.log("unknown OSC address", address);
        break;
    }
  }

  set recordingList(filenames: string[]) {
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
          `http://localhost:8083/${filename}`,
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
    const offset = t2 - t1 + owd;
    this.offsets.push(offset);
    if (this.offsets.length > 20) {
      this.offsets.shift();
    }
    this.offsetAvergae =
      this.offsets.reduce((a, b) => a + b) / this.offsets.length;
  }

  _requestSync() {
    if (!this.ws) return;
    const now = Date.now();
    const msg = { message_type: "sync", data: `${now}` };
    this.ws.send(JSON.stringify(msg));
  }
}
