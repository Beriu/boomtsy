import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
    
    root: __dirname + '/app',
    
    plugins: [vue()],
    
    server: {
        port: 8080
    },

    build: {
        outDir: __dirname + '/distApp'
    }
});