class AudioProcessor extends AudioWorkletProcessor {
    constructor(nodeOptions) {
        super();
        this.buffer = new CircularBuffer(2048);
        this.port.onmessage = async event => {
            if (event.data.type === "buffer") {
                this.buffer.write(event.data.buffer);
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
        this.reads = 0;
        this.writes = 0;
    }

    clear() {
        this.buffer.fill(0);
    }

    write(values) {
        this.writes += 1;
        let end = this.writeCursor + values.length;
        if (end < this.buffer.length) {
            if (this.writeCursor < this.readCursor && this.readCursor <= end) {
                // Don't write over slice that is about to be read, skip this audio
                return;
            }
            this.buffer.set(values, this.writeCursor);
        } else {
            end = end - this.buffer.length;
            if (this.writeCursor < this.readCursor && this.readCursor <= end) {
                // Don't write over slice that is about to be read, skip this audio
                return;
            }
            const cut = this.writeCursor - values.length
            this.buffer.set(values.subarray(0, cut), this.writeCursor);
            this.buffer.set(values.subarray(cut), 0);
        }
        this.writeCursor = end;
    }

    readSlice(length) {
        this.reads += 1;
        // console.log("Reads, writes", this.reads / this.writes);
        let end = this.readCursor + length;
        let result = null;
        if (end < this.buffer.length) {
            result = this.buffer.subarray(this.readCursor, end);
        } else {
            end = end - this.buffer.length;
            result = new Float32Array([
                ...this.buffer.subarray(this.readCursor),
                ...this.buffer.subarray(0, end)
            ]);
        }
        this.readCursor = end;
        return result;
    }
}