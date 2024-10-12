export default class GranularSynth {
  ctx?: AudioContext;
  granularSynth?: AudioWorkletNode;

  async init(ctx: AudioContext) {
    this.ctx = ctx;
    await this.ctx.audioWorklet.addModule("module.js");
    //gain?.linearRampToValueAtTime(0, this.ctx.currentTime + 2);
    setInterval(() => {
      this.play();
    }, 100);
  }

  play() {
    if (!this.ctx) return;
    const granularSynth = new AudioWorkletNode(this.ctx, "granular-synth");
    granularSynth.connect(this.ctx.destination);
    granularSynth.port.postMessage({ type: "init" });
    const gain = granularSynth.parameters.get("gain");
    gain?.setValueAtTime(0, this.ctx.currentTime);
    gain?.linearRampToValueAtTime(0.3, this.ctx.currentTime + 1);
  }
}
