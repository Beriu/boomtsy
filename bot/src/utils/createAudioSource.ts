import {createAudioResource, demuxProbe, AudioResource} from "@discordjs/voice";
import ytdl, { type downloadOptions } from "ytdl-core";
import { Song } from "../types";

export default async function createAudioSource(song: Song): Promise<AudioResource<Song>> {
    
    const options: downloadOptions = {
        requestOptions: {
            headers: {
                "X-Youtube-Identity-Token": process.env.YOUTUBE_TOKEN
            }
        },
        filter: 'audioonly',
        highWaterMark: 1 << 25,
        quality: 'highestaudio'
    };

    const streamSource = ytdl(song.url, options);
    const { stream, type: inputType } = await demuxProbe(streamSource);
    return createAudioResource(stream, { inputType, metadata: song });
}