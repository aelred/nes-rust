export default {
    root: "./web",
    server: {
        open: true,
        port: 8080,
        cors: true,
        headers: {
            'Cross-Origin-Opener-Policy': 'same-origin',
            'Cross-Origin-Embedder-Policy': 'require-corp',
        },
    },
}