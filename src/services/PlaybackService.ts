import youtubeDecoder from 'ytdl-core';
import {interactionPayload} from "../types";

export default class PlaybackService {

    async handle(url: string) {
        console.log(url);
    }
}