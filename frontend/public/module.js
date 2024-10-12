class GranularSynth extends AudioWorkletProcessor {
  constructor() {
    super();
    this.port.onmessage = this.messageHandler.bind(this);
    this.isActive = true;
    this.count = 0;
  }

  process(_, outputList, parameters) {
    const outputs = outputList[0];
    const gain = parameters.gain;

    this.count++;
    if (this.count >= 1000) {
      this.isActive = false;
    }

    for (let channel = 0; channel < outputs.length; channel++) {
      const output = outputs[channel];
      for (let i = 0; i < output.length; i++) {
        output[i] = Math.random() - 0.5;
        output[i] *= gain[i];
      }
    }

    return this.isActive;
  }

  static get parameterDescriptors() {
    return [
      {
        name: "gain",
        defaultValue: 0,
        minValue: 0,
        maxValue: 1,
      },
      {
        name: "frequency",
        defaultValue: 440.0,
        minValue: 27.5,
        maxValue: 4186.009,
      },
    ];
  }

  #setArrayBuffer(arrayBuffer) {
    this.arrayBuffer = arrayBuffer;
  }

  messageHandler(event) {
    //console.log(event.data);
  }
}

registerProcessor("granular-synth", GranularSynth);
