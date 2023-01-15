import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
    root: __dirname + '/dashboard',
    plugins: [vue()],
    build: {
        outDir: __dirname + '/distApp'
    }
});