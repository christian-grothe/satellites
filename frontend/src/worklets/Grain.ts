export class Trigger {
  phase: number;
  increment: number;

  constructor() {
    this.phase = 0;
    this.increment = 10 / 48000;
  }

  setInc(inc: number) {
    this.increment = inc / 48000;
  }

  tick(): boolean {
    this.phase += this.increment;
    if (this.phase >= 1.0) {
      this.phase -= 1.0;
      return true;
    }

    return false;
  }
}

export class GrainEnvelope {
  inc: number;
  phase!: number;
  sin0!: number;
  sin1!: number;
  dsin!: number;

  constructor(buffersize: number) {
    this.inc = 1 / buffersize;
    this.reset();
  }

  reset() {
    this.phase = 0.0;
    this.sin0 = Math.sin(this.phase * Math.PI);
    this.sin1 = Math.sin((this.phase - this.inc) * Math.PI);
    this.dsin = 2.0 * Math.cos(this.inc * Math.PI);
  }

  setInc(inc: number) {
    this.inc = inc;
    this.reset();
  }

  tick(): number {
    const sinx = this.dsin * this.sin0 - this.sin1;
    this.sin1 = this.sin0;
    this.sin0 = sinx;
    return sinx;
  }
}

export class Grain {
  env: GrainEnvelope;
  active: boolean;
  pos: number;
  pitch: number;
  buffersize: number;
  dur: number;
  age: number;

  constructor(buffersize: number) {
    this.env = new GrainEnvelope(buffersize);
    this.buffersize = buffersize;
    this.active = false;
    this.pos = 0;
    this.pitch = 1;
    this.dur = 0;
    this.age = 0;
  }

  activate(start: number, pitch: number, dur: number) {
    this.pos = start;
    this.pitch = pitch;
    this.active = true;
    this.dur = dur;
    this.age = 0;
    this.env.setInc(1 / dur);
  }

  tick() {
    this.pos += this.pitch;
    this.age += 1;
    const gain = this.env.tick();
    // grain is finished once its windowed lifetime has elapsed; free the slot
    if (this.age >= this.dur) {
      this.active = false;
    }
    return { pos: this.pos, gain };
  }
}
