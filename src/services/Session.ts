import {Song} from "../types";
import {VoiceChannel} from "discord.js";
import {AudioPlayer, AudioPlayerStatus, createAudioPlayer, entersState, NoSubscriberBehavior, VoiceConnection, VoiceConnectionStatus} from "@discordjs/voice";
import createVoiceConnection from "../utils/createVoiceConnection";
import createAudioSource from "../utils/createAudioSource";
import Queue from "./Queue";

interface StatusHaver {
    status: string;
}

const printStateChangeToConsole = (identifier: string) => {
    return (oldState: StatusHaver, newState: StatusHaver) => {

        const now = new Date();
        const nowToString = `${now.getHours()}:${now.getMinutes()}:${now.getSeconds()}`;

        console.info(`[${nowToString}][${identifier}][${oldState.status}->${newState.status}]`);
    }
};

export default class Session {

    private readonly player: AudioPlayer;
    private readonly connection: VoiceConnection;

    private queue: Queue<Song> = new Queue();
    private current: Song | null = null;

    constructor(voiceChannel: VoiceChannel) {

        this.player = createAudioPlayer({ behaviors: { noSubscriber: NoSubscriberBehavior.Stop } });
        this.connection = createVoiceConnection(voiceChannel);
        
        entersState(this.connection, VoiceConnectionStatus.Ready, 30_000)
            .then(() => { this.connection.subscribe(this.player); });

        this.player.on('error', console.error);
        
        this.connection.on('stateChange', printStateChangeToConsole('VoiceConnection'));
        this.player.on('stateChange', printStateChangeToConsole('AudioPlayer'));

        this.player.on(AudioPlayerStatus.Idle, async (oldState, newState) => {

            if(this.queue.peek()) {
                this.current = this.queue.dequeue()!;
                this.player.play(
                    await createAudioSource(this.current)
                );
            }else{
                this.current = null;
                this.player.stop();
            }
        });
    }

    isNotPlaying() {
        return this.player.state.status !== AudioPlayerStatus.Playing;
    }

    addSong(song: Song) {
        this.queue.enqueue(song);
    }

    async start() {
        if(!this.current) {
            this.current = this.queue.dequeue()!;
        }
        this.player.play(await createAudioSource(this.current));
    }

    async next() {
        if(this.queue.peek()) {
            this.player.stop(true);
            this.current = this.queue.dequeue()!;
            this.player.play(await createAudioSource(this.current));
            return this.current;
        }
        return false;
    }

    stop() {
        this.player.stop(true);
        this.connection.destroy();
    }

    songs() {
        return {
            current: this.current,
            queue: this.queue.getItems()
        };
    }
}