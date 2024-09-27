class AudioProcessor extends AudioWorkletProcessor {
    constructor(nodeOptions) {
        super();
        this.buffer = new CircularBuffer(2048);
        this.port.onmessage = event => {
            if (event.data.type === "buffer") {
                for (const value of event.data.buffer) {
                    this.buffer.write(value);
                }
            } else if (event.data.type === "mute") {
                console.log("Muting");
                this.buffer.clear();
            }
        }
    }

    process(inputs, outputs, parameters) {
        outputs[0][0].set(this.buffer.readSlice(outputs[0][0].length));
        return true;
    }
}

registerProcessor("audio-processor", AudioProcessor);

class CircularBuffer {
    constructor(size) {
        this.buffer = new Float32Array(size);
        this.writeCursor = Math.floor(size / 2);
        this.readCursor = 0;
    }

    clear() {
        this.buffer.fill(0);
    }

    write(value) {
        this.buffer[this.writeCursor] = value;
        this.writeCursor = (this.writeCursor + 1) % this.buffer.length;
    }

    readSlice(length) {
        let end = this.readCursor + length;
        let result = null;
        if (end < this.buffer.length) {
            result = this.buffer.slice(this.readCursor, end);
        } else {
            end = end - this.buffer.length;
            result = new Float32Array([
                ...this.buffer.slice(this.readCursor),
                ...this.buffer.slice(0, end)
            ]);
        }
        this.readCursor = end;
        return result;
    }
}