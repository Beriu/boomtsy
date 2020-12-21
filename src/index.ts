import ytdl from 'ytdl-core';
import {Readable} from "stream";

const audio: Readable = ytdl(
    'https://www.youtube.com/watch?v=5gHdZkavGJk',
    { quality: 'highestaudio', filter: 'audioonly' }
    );

/**
 * TODO: Create broadcast
 * https://github.com/fent/node-ytdl-core
 * https://discord.js.org/#/docs/main/stable/class/Client
 */