import WasmPackPlugin from '@wasm-tool/wasm-pack-plugin';
import CopyWebpackPlugin from 'copy-webpack-plugin';
import HtmlWebpackPlugin from 'html-webpack-plugin';
import { resolve } from 'path';
const __dirname = import.meta.dirname;

export default {
    entry: './pkg/index.js',
    output: {
        path: resolve(__dirname, 'dist'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: 'index.html'
        }),
        new CopyWebpackPlugin({
            patterns: [
                { from: 'audio-processor.js', to: 'audio-processor.js' },
            ]
        }),
        new WasmPackPlugin({
            crateDirectory: resolve(__dirname, '..'),
            outDir: resolve(__dirname, 'pkg'),
            forceMode: 'production',
            extraArgs: '--no-default-features --features=web',
        }),
    ],
    mode: 'development',
    experiments: {
        asyncWebAssembly: true
    },
    devServer: {
        headers: {
            // Required to run locally without CORS problems
            'Cross-Origin-Opener-Policy': 'same-origin',
            'Cross-Origin-Embedder-Policy': 'require-corp'
        }
    }
}