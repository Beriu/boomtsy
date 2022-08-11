import {createAudioResource, demuxProbe, AudioResource} from "@discordjs/voice";
import ytdl from "ytdl-core";

export default async function createAudioSource(url: string): Promise<AudioResource<null>> {

    const { stream, type: inputType } = await demuxProbe(
        ytdl(
            url,
            { filter: 'audioonly', highWaterMark: 1 << 30, quality: 'highestaudio' }
        )
    );
    return createAudioResource(stream, { inputType });
}