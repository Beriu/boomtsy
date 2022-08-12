import {Song} from "../types";
import {GuildChannel} from "discord.js";
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

    constructor(guildChannel: GuildChannel) {

        this.player = createAudioPlayer({ debug: true, behaviors: { noSubscriber: NoSubscriberBehavior.Stop } });
        this.connection = createVoiceConnection(guildChannel);
        
        entersState(this.connection, VoiceConnectionStatus.Ready, 30_000)
            .then(() => { this.connection.subscribe(this.player); });

        this.player.on('error', console.error);
        this.connection.on('error', console.error);
        this.connection.on('debug', console.info);
        
        this.connection.on('stateChange', printStateChangeToConsole('VoiceConnection'));
        this.player.on('stateChange', printStateChangeToConsole('AudioPlayer'));

        this.player.on(AudioPlayerStatus.AutoPaused, async (oldState, newState) => {
            if(!this.queue.isEmpty()) {
                this.current = this.queue.dequeue()!;
                this.player.play(
                    await createAudioSource(this.current.url)
                );
            }
        });
    }

    isNotPlaying() {
        return this.player.state.status !== AudioPlayerStatus.Playing;
    }

    addSong(song: Song) {
        console.info('Add command', `Queue size: ${this.queue.size()}`);
        this.queue.enqueue(song);
    }

    async start() {
        console.info('Start command', `Queue size: ${this.queue.size()}`);
        if(!this.current) {
            this.current = this.queue.dequeue()!;
        }
        this.player.play(await createAudioSource(this.current.url));
    }

    async next() {
        console.info('Next command', `Queue size: ${this.queue.size()}`);
        if(!this.queue.isEmpty()) {
            this.player.stop(true);
            this.current = this.queue.dequeue()!;
            this.player.play(await createAudioSource(this.current.url));
            return this.current;
        }
        return false;
    }

    stop() {
        console.info('Stop command', `Queue size: ${this.queue.size()}`);
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