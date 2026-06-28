// @types/audioworklet does not ship AudioParamDescriptor, so declare it locally.
interface AudioParamDescriptor {
  name: string;
  defaultValue?: number;
  minValue?: number;
  maxValue?: number;
  automationRate?: "a-rate" | "k-rate";
}

class Env {
  val = 0.0;
  inc = 0.0;
  state: "att" | "rel" | "hold" | "off" = "hold";
  loop = false;

  constructor(incSec: number, loop: boolean) {
    this.val = 0;
    this.inc = 1 / (48000 * incSec);
    this.state = "att";
    this.loop = loop;
  }

  tick() {
    switch (this.state) {
      case "att": {
        this.val += this.inc;
        if (this.val >= 1.0) {
          if (this.loop) {
            this.state = "hold";
          } else {
            this.state = "rel";
          }
        }
        break;
      }
      case "rel": {
        this.val -= this.inc;
        if (this.val <= 0.0 && !this.loop) {
          this.state = "off";
        } else if (this.val <= 0.0 && this.loop) {
          this.state = "att";
        }
        break;
      }
      case "hold":
        break;
    }
  }
}

class GranularSynth extends AudioWorkletProcessor {
  private isActive = true;
  private buffer: AudioBuffer | null = null;
  private playhead = 0;
  private env = new Env(0.1, true);
  private mstrEnv = new Env(5.0, false);
  private pitch = 1.5;
  private spray = 0.0005;
  private loopstart = 0.3;
  private looplengths = 0.05;

  constructor() {
    super();
    this.port.onmessage = this.messageHandler.bind(this);
  }

  process(
    _inputs: Float32Array[][],
    outputs: Float32Array[][],
    // _parameters: Record<string, Float32Array>,
  ): boolean {
    const output = outputs[0];
    if (!this.buffer || !this.isActive) return this.isActive;

    for (let channel = 0; channel < output.length; channel++) {
      const samples = output[channel];
      for (let i = 0; i < samples.length; i++) {
        const intIdx = Math.floor(this.playhead);
        const nextIdx = (intIdx + 1) % this.buffer.length;
        const frac = this.playhead - intIdx;

        const nextSample =
          this.buffer[intIdx] * (1.0 - frac) + this.buffer[nextIdx] * frac;

        samples[i] = nextSample * this.env.val * this.mstrEnv.val;
        this.playhead += this.pitch;

        this.env.tick();
        this.mstrEnv.tick();

        const remainingSamples = this.buffer.length - this.playhead;
        const remaingingTime = remainingSamples / this.pitch;
        const releaseTime = 1 / this.env.inc;
        if (remaingingTime <= releaseTime) {
          this.env.state = "rel";
        }

        if (
          this.playhead >=
          this.buffer.length * (this.loopstart + this.looplengths) ||
          this.playhead >= this.buffer.length
        ) {
          const rand = Math.random() * this.spray;
          this.loopstart = this.loopstart + rand;
          this.playhead = this.buffer.length * this.loopstart;
        }

        if (this.mstrEnv.state === "off") {
          this.isActive = false;
        }
      }
    }

    return this.isActive;
  }

  static get parameterDescriptors(): AudioParamDescriptor[] {
    return [
      { name: "gain", defaultValue: 0, minValue: 0, maxValue: 1 },
      {
        name: "frequency",
        defaultValue: 440,
        minValue: 27.5,
        maxValue: 4186.009,
      },
    ];
  }

  private messageHandler(event: MessageEvent) {
    this.buffer = event.data.buffer;
    this.playhead = this.loopstart * event.data.buffer.length;
  }
}

registerProcessor("granular-synth", GranularSynth);
