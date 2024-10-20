interface PlayMessage {
  pitch: number;
  att: number;
  rel: number;
  hold: number;
  gain: number;
  start: number;
  length: number;
  bufIndex: number;
  timestamp: number;
}

class Sampler {
  ctx: AudioContext | undefined;
  mstrVol: GainNode | undefined;
  source: AudioBufferSourceNode | undefined;
  sampleLength: number;
  _recordingList: string[];
  _audioBuffers: AudioBuffer[];

  constructor() {
    this.sampleLength = 0;
    this._recordingList = [];
    this._audioBuffers = [];
  }

  init(ctx: AudioContext, mstrVol: GainNode) {
    this.ctx = ctx;
    this.mstrVol = mstrVol;
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
    this._audioBuffers = [];
    try {
      for (const filename of this._recordingList) {
        if (filename === "test.wav") continue;
        const response = await fetch(
          `http://satellites.local:8080/recordings/${filename}`,
        );
        const arrayBuffer = await response.arrayBuffer();
        const newBuffer = await this.ctx.decodeAudioData(arrayBuffer);
        this._audioBuffers.push(newBuffer);
      }
    } catch (e) {
      console.log(e);
    }
  }

  setAndPlay(argsArray: any[], offset: number, buffers: AudioBuffer[]) {
    let args: { [key: string]: any } = {};
    for (let i = 0; i < argsArray.length; i += 2) {
      const key = argsArray[i];
      const value = argsArray[i + 1];
      args[key] = value;
    }
    const currentTime = Date.now() + offset;
    const delay = args.timestamp - currentTime;

    this.play(buffers, delay, args as PlayMessage);
  }

  play(buffers: AudioBuffer[], delay: number, args: PlayMessage) {
    if (!this.ctx || !this.mstrVol) return;
    const currentTime = this.ctx.currentTime + delay / 1000;

    const { att, rel, hold, gain, pitch, start, length, bufIndex } = args;

    const totalLength = att + rel + hold;

    const source = this.ctx.createBufferSource();
    const gainNode = this.ctx.createGain();
    const mstr = this.ctx.createGain();

    source.connect(gainNode);
    gainNode.connect(mstr);
    mstr.connect(this.mstrVol);
    mstr.gain.setValueAtTime(gain, currentTime);

    gainNode.gain.setValueAtTime(0, currentTime);
    gainNode.gain.linearRampToValueAtTime(1, currentTime + att);
    gainNode.gain.linearRampToValueAtTime(1, currentTime + att + hold);
    gainNode.gain.linearRampToValueAtTime(0, currentTime + totalLength);

    source.buffer = buffers[bufIndex % buffers.length];

    this.sampleLength = source.buffer.duration;

    source.playbackRate.value = pitch;
    source.loop = true;
    source.loopStart = this.sampleLength * start;
    source.loopEnd = this.sampleLength * start + length;

    source.start(currentTime, source.loopStart);
    source.stop(currentTime + totalLength);

    source.onended = () => {
      source.disconnect();
      gainNode.disconnect();
    };
  }
}

export default Sampler;
