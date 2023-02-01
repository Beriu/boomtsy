import { defineConfig, loadEnv } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig(({ command, mode }) => {

    // Load env file based on `mode` in the current working directory.
    // Set the third parameter to '' to load all env regardless of the `VITE_` prefix.
    const env = loadEnv(mode, process.cwd(), '')

    return {
        
        define: {
            __APP_ENV__: env.APP_ENV,
        },

        root: __dirname + '/app',

        plugins: [vue()],

        server: {
            port: 8080,
            proxy: {
                "^/api.*": {
                    target: `http://localhost:${env.WEBSERVER_PORT}`,
                    changeOrigin: true,
                },
                '/socket.io': {
                    target: 'ws://localhost:5000',
                    ws: true,
                }
                
            }
        },

        build: {
            outDir: __dirname + '/distApp'
        }
    }
})