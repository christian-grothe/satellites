import workletUrl from "./worklets/granular-synth.worklet.ts?worker&url";

export default class GranularSynth {
  ctx?: AudioContext;
  granularSynth?: AudioWorkletNode;

  async init(ctx: AudioContext) {
    this.ctx = ctx;
    await this.ctx.audioWorklet.addModule(workletUrl);
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

  setAndPlay(audioBuffers: AudioBuffer[]) {
    if (!this.ctx) return;
    const granularSynth = new AudioWorkletNode(this.ctx, "granular-synth");
    granularSynth.connect(this.ctx.destination);
    granularSynth.port.postMessage({
      type: "init",
      buffer: audioBuffers[0].getChannelData(0),
    });
    const gain = granularSynth.parameters.get("gain");
    gain?.setValueAtTime(0, this.ctx.currentTime);
    gain?.linearRampToValueAtTime(0.3, this.ctx.currentTime + 1);
  }
}
