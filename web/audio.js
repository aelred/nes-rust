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
    if (audioProcessorNode && document.visibilityState === 'hidden') {
        audioProcessorNode.port.postMessage({ type: "mute" });
    }
});

export function pushAudioBuffer(value) {
    if (audioProcessorNode == null) return;
    buffer[bufferIndex] = value - 0.5;
    bufferIndex += 1;
    if (bufferIndex === BUFFER_SIZE) {
        bufferIndex = 0;

        // Silence audio when page isn't visible
        const event = document.visibilityState === 'visible' ? { type: "buffer", buffer } : { type: "mute" };
        audioProcessorNode.port.postMessage(event);
    }
}
