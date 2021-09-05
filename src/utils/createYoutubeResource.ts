import {createAudioResource} from "@discordjs/voice";
import ytdl from "ytdl-core";

export default function createYoutubeResource(url: string) {
    return createAudioResource(ytdl(url, { filter: 'audioonly' }));
}