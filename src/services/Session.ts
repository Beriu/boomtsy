import {createAudioPlayer, createAudioResource, VoiceConnection} from "@discordjs/voice";
import ytdl from "ytdl-core";
import {Song} from "../types";
import {VoiceChannel} from "discord.js";
import createVoiceConnection from "../utils/createVoiceConnection";
import createYoutubeResource from "../utils/createYoutubeResource";

export default class Session {

    private player = createAudioPlayer();
    private connection?: VoiceConnection;
    private queue: Song[] = [];
    private nextIndex: number = -1;

    constructor() {
        this.player.on("stateChange", (oldState, newState) => {
            if(newState.status === 'autopaused') {
                if(this.nextIndex !== -1 && this.queue[this.nextIndex]) {
                    const song = this.queue[this.nextIndex];
                    this.player.play(createYoutubeResource(song.url));
                }
            }
        });
    }

    isNotPlaying() {
        return this.player.state.status !== 'playing';
    }

    addSong(song: Song) {
        this.queue.push(song);
        this.nextIndex++;
    }

    removeSong(index: number) {
        this.queue.splice(index, 1);
    }

    start(voiceChannel: VoiceChannel) {
        if(!this.connection) {
            this.connection = createVoiceConnection(voiceChannel);
        }
        const song = this.queue[this.nextIndex];
        this.player.play(createYoutubeResource(song.url));
        this.connection.subscribe(this.player);
    }

    next() {
        if(this.nextIndex <= this.queue.length) {
            this.player.stop(true);
            const song = this.queue[this.nextIndex];
            this.player.play(createYoutubeResource(song.url));
            return song;
        }
        this.stop();
        return false;
    }

    stop() {
        this.queue = [];
        this.nextIndex = -1;
        this.player.stop(true);
        this.connection?.destroy();
    }
}