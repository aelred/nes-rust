const BUFFER_SIZE = 128;

let audioProcessorNode = null;
const buffer = new Float32Array(BUFFER_SIZE);
let bufferIndex = 0;

async function startAudio() {
    if (audioProcessorNode) return;
    const context = new AudioContext({ sampleRate: 44100 });
    await context.audioWorklet.addModule('./audio-processor.js');
    audioProcessorNode = new AudioWorkletNode(context, 'audio-processor');
    audioProcessorNode.connect(context.destination);
}

addEventListener("click", startAudio);
addEventListener("keydown", startAudio);
addEventListener("visibilitychange", () => {
    if (audioProcessorNode && document.visibilityState !== 'visible') {
        audioProcessorNode.port.postMessage(new Float32Array(BUFFER_SIZE))
    }
});

export function pushAudioBuffer(byte) {
    if (audioProcessorNode == null || document.visibilityState !== 'visible') return;
    buffer[bufferIndex] = (byte / 255) - 0.5;
    bufferIndex += 1;
    if (bufferIndex === BUFFER_SIZE) {
        bufferIndex = 0;
        audioProcessorNode.port.postMessage(buffer);
    }
}
