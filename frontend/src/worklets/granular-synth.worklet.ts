import { Grain, Trigger } from "./Grain";

const NUM_GRAIN = 16;

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
  incAtt = 0.0;
  incRel = 0.5;
  state: "att" | "rel" | "hold" | "off" = "hold";
  loop = false;

  constructor(incAtt: number, incRel: number, loop: boolean) {
    this.val = 0;
    this.incAtt = 1 / (48000 * incAtt);
    this.incRel = 1 / (48000 * incRel);
    this.state = "att";
    this.loop = loop;
  }

  tick() {
    switch (this.state) {
      case "att": {
        this.val += this.incAtt;
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
        this.val -= this.incRel;
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
  private buffer: Float32Array | null = null;
  private playhead = 0;
  private env = new Env(0.005, 0.005, true);
  private mstrEnv = new Env(0.0, 0.5, false);
  private pitch = 1;
  private spray = 0.0;
  private loopstart = 0;
  private looplength = 0.1;
  private grainLength = 0.25;
  private grains!: Grain[];
  private trigger!: Trigger;
  private mode: "tape" | "grain" = "tape";

  constructor() {
    super();
    this.port.onmessage = this.messageHandler.bind(this);
    this.trigger = new Trigger();
  }

  process(
    _inputs: Float32Array[][],
    outputs: Float32Array[][],
    // _parameters: Record<string, Float32Array>,
  ): boolean {
    const output = outputs[0];
    if (!this.buffer || !this.isActive) return this.isActive;

    // left channel
    const samples = output[0];
    for (let i = 0; i < samples.length; i++) {
      if (this.mode === "grain") {
        // 1. activate new grain
        if (this.trigger.tick()) {
          for (const grain of this.grains) {
            if (!grain.active) {
              grain.activate(
                this.playhead,
                this.pitch,
                48000 * this.grainLength,
              );
              break;
            }
          }
        }

        // 2. gather grain data
        const grainData = this.grains
          .filter((grain) => grain.active)
          .map((grain) => grain.tick());

        // 3. apply grain
        for (const grain of grainData) {
          const floorPos = Math.floor(grain.pos);
          const intIdx = floorPos % this.buffer.length;
          const nextIdx = (intIdx + 1) % this.buffer.length;
          const frac = grain.pos - floorPos;

          const nextSample =
            this.buffer[intIdx] * (1.0 - frac) + this.buffer[nextIdx] * frac;

          samples[i] += nextSample * this.mstrEnv.val * grain.gain;
        }

        this.playhead += 1;
      } else {
        const intIdx = Math.floor(this.playhead);
        const nextIdx = (intIdx + 1) % this.buffer.length;
        const frac = this.playhead - intIdx;

        const nextSample =
          this.buffer[intIdx] * (1.0 - frac) + this.buffer[nextIdx] * frac;
        samples[i] = nextSample * this.env.val * this.mstrEnv.val;

        this.playhead += this.pitch;
        this.env.tick();
      }

      this.mstrEnv.tick();

      // Grain window: trigger the release so the per-grain envelope reaches 0
      // right at the loop-wrap point — not at the end of the whole buffer.
      const grainEnd = this.buffer.length * (this.loopstart + this.looplength);
      const remainingGrainSamples = (grainEnd - this.playhead) / this.pitch;
      const releaseTime = 1 / this.env.incRel;
      if (remainingGrainSamples <= releaseTime) {
        this.env.state = "rel";
      }

      if (this.playhead >= grainEnd || this.playhead >= this.buffer.length) {
        const rand = Math.random() * 2.0 - 1.0;
        this.loopstart = this.loopstart + rand * this.spray;
        this.playhead = this.buffer.length * this.loopstart;
        // restart the window from silence so the fade-in begins at the new grain
        this.env.state = "att";
        this.env.val = 0;
      }

      if (this.mstrEnv.state === "off") {
        this.isActive = false;
      }
    }

    return this.isActive;
  }

  static get parameterDescriptors(): AudioParamDescriptor[] {
    return [{ name: "gain", defaultValue: 0, minValue: 0, maxValue: 1 }];
  }

  private messageHandler(event: MessageEvent) {
    this.buffer = event.data.buffer;
    this.playhead = this.loopstart * event.data.buffer.length;

    this.loopstart = event.data.loopstart || 0.0;
    this.playhead = this.loopstart * event.data.buffer.length;
    this.looplength = event.data.looplength || 0.01;

    this.spray = event.data.spray || 0.0;

    this.mstrEnv.incAtt = 1 / (48000 * (event.data.incAtt || 0.01));
    this.mstrEnv.incRel = 1 / (48000 * (event.data.incRel || 1));
    this.mode = "grain";

    this.pitch = event.data.pitch || 1.0;
    this.grainLength = event.data.length || 0.25;

    this.trigger.setInc(event.data.dens || 10);
    this.grains = Array.from(
      { length: NUM_GRAIN },
      () => new Grain(event.data.buffer.length),
    );
  }
}

registerProcessor("granular-synth", GranularSynth);
