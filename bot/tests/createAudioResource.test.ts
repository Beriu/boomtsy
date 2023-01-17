import createAudioSource from "../src/utils/createAudioSource";
import { createWriteStream } from "fs";

test('youtube download.', async () => {

    const source = await createAudioSource('http://www.youtube.com/watch?v=aqz-KE-bpKQ')
    source.playStream.pipe(createWriteStream(__dirname + '/testData/dump.webm'));
});