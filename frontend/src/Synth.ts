export class Synth {
  ctx: AudioContext | undefined;
  mstrVol: GainNode | undefined;

  init(ctx: AudioContext, mstrVol: GainNode) {
    this.ctx = ctx;
    this.mstrVol = mstrVol;
  }

  play() {
    console.log("Synth.play()");
    if (!this.ctx || !this.mstrVol) return;
    const osc = this.ctx.createOscillator();
    osc.connect(this.mstrVol);
    osc.start();
    osc.stop(this.ctx.currentTime + 1);
    osc.onended = () => {
      osc.disconnect();
    };
  }
}
