import workletUrl from "./worklets/granular-synth.worklet.ts?worker&url";

export default class GranularSynth {
  ctx?: AudioContext;
  granularSynth?: AudioWorkletNode;
  ready = false;

  async init(ctx: AudioContext) {
    this.ctx = ctx;
    await this.ctx.audioWorklet.addModule(workletUrl);
    this.ready = true;
    // gain?.linearRampToValueAtTime(0, this.ctx.currentTime + 2);
    // this.play();
    // setInterval(() => {
    // }, 1000);
  }

  play() {
    if (!this.ctx) return;
    const granularSynth = new AudioWorkletNode(this.ctx, "granular-synth");
    granularSynth.connect(this.ctx.destination);
    granularSynth.port.postMessage({ type: "init", pet: "dog" });
    const gain = granularSynth.parameters.get("gain");
    gain?.setValueAtTime(0, this.ctx.currentTime);
    gain?.linearRampToValueAtTime(0.3, this.ctx.currentTime + 1);
  }

  setAndPlay(
    audioBuffers: AudioBuffer[],
    argsArray: (string | number)[],
    offset: number,
  ) {
    if (!this.ctx || !this.ready) return;

    const args: { [key: string]: number } = {};
    for (let i = 0; i < argsArray.length; i += 2) {
      const key = argsArray[i] as string;
      const value = argsArray[i + 1] as number;
      args[key] = value;
    }

    const {
      loopstart,
      looplength,
      spray,
      incAtt,
      incRel,
      pitch,
      length,
      dens,
      timestamp,
      beat,
      mode
    } = args;

    // Convert the server-clock event time into an absolute time on THIS
    // device's audio clock. The worklet then waits for that instant using its
    // own sample counter, so the postMessage/render-quantum latency between
    // here and the audio thread no longer smears the start time.
    const serverNow = performance.timeOrigin + performance.now() + offset;
    const delayMs = timestamp + beat * 1000 - serverNow;
    const startTime = this.ctx.currentTime + delayMs / 1000;

    const granularSynth = new AudioWorkletNode(this.ctx, "granular-synth");
    granularSynth.connect(this.ctx.destination);
    granularSynth.port.postMessage({
      type: "init",
      buffer: audioBuffers[0].getChannelData(0),
      loopstart,
      looplength,
      incAtt,
      incRel,
      pitch,
      spray,
      length,
      dens,
      startTime,
      mode,
    });
  }
}
