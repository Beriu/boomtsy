import {createAudioResource, demuxProbe, AudioResource} from "@discordjs/voice";
import ytdl from "ytdl-core";

export default async function createAudioSource(url: string): Promise<AudioResource<null>> {

    const requestOptions = { 
        headers: {
            "X-Youtube-Identity-Token": process.env.YOUTUBE_TOKEN
        } 
    };
    const funnel = ytdl(
        url,
        { requestOptions, filter: 'audioonly', highWaterMark: 1 << 25, quality: 'highestaudio' }
    );

    const { stream, type: inputType } = await demuxProbe(funnel);
    return createAudioResource(stream, { inputType });
}