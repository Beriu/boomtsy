import {Song} from "../types";
import {GuildChannel, VoiceChannel} from "discord.js";
import {AudioPlayer, AudioPlayerStatus, createAudioPlayer, VoiceConnection} from "@discordjs/voice";
import createVoiceConnection from "../utils/createVoiceConnection";
import createAudioSource from "../utils/createAudioSource";

export default class Session {

    private readonly player: AudioPlayer;
    private readonly connection: VoiceConnection;
    private queue: Song[] = [];
    private nextIndex: number = 0;

    constructor(guildChannel: GuildChannel) {

        this.player = createAudioPlayer();
        this.connection = createVoiceConnection(guildChannel);

        this.player.on('error', console.error);
        this.connection.on('error', console.error);

        this.player.on("stateChange", (oldState, newState) => {
            if(newState.status === AudioPlayerStatus.AutoPaused) {
                if(this.nextIndex !== -1 && this.queue[this.nextIndex]) {
                    const song = this.queue[this.nextIndex];
                    this.player.play(createAudioSource(song.url));
                }
            }
        });
    }

    isNotPlaying() {
        return this.player.state.status !== AudioPlayerStatus.Playing;
    }

    addSong(song: Song) {
        this.queue.push(song);
    }

    removeSong(index: number) {
        this.queue.splice(index, 1);
    }

    start() {
        const song = this.queue[this.nextIndex];
        this.player.play(createAudioSource(song.url));
        this.connection.subscribe(this.player);
        this.nextIndex++;
    }

    next() {
        if(this.nextIndex <= this.queue.length) {
            this.player.stop(true);
            const song = this.queue[this.nextIndex];
            this.player.play(createAudioSource(song.url));
            return song;
        }
        this.stop();
        return false;
    }

    stop() {
        this.player.stop(true);
        this.connection.destroy();
    }
}