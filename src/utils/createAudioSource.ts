import {createAudioResource} from "@discordjs/voice";
import ytdl from "ytdl-core";

export default function createAudioSource(url: string) {
    return createAudioResource(
        ytdl(url, { filter: 'audioonly', highWaterMark: 1 << 25, quality: 'highestaudio' })
    );
}