/**
 * Simple ringbuffer implementation to continuously feed a stream of samples to the node's output
 */
class RingBufferQueueSource extends AudioWorkletProcessor {
  constructor({ processorOptions: { capacity } }) {
    super();
    this.buffer = new Float32Array(capacity);
    this.readIndex = 0;
    this.writeIndex = 0;
    this.hasSamples = false;

    this.port.onmessage = this.onmessage.bind(this);
  }

  onmessage(e) {
    const { data } = e;
    const i = this.writeIndex % this.buffer.length;
    if (this.writeIndex + data.length >= this.buffer.length) {
      // Copy in two parts
      this.buffer.set(data.slice(0, this.buffer.length - i), i);
      this.buffer.set(data.slice(this.buffer.length - i, data.length), 0);
    } else {
      // Copy just once
      this.buffer.set(data, i);
    }
    this.writeIndex += data.length;
    // this.writeIndex = (this.writeIndex + data.length) % this.buffer.length;
    this.hasSamples = true;
  }

  process(inputs, outputs) {
    const output = outputs[0];
    output.forEach((channel) => {
      for (let i = 0; i < channel.length; i++) {
        if (this.readIndex < this.writeIndex) {
          channel[i] = this.buffer[this.readIndex % this.buffer.length];
          // Cycle around
          this.readIndex += 1;
        } else {
          channel[i] = this.buffer[this.readIndex % this.buffer.length];
        }
      }
    });
    return true;
  }
}

registerProcessor("ring-buffer-queue-source", RingBufferQueueSource);
