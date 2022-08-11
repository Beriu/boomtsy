import {Song} from "../types";
import {GuildChannel} from "discord.js";
import {AudioPlayer, AudioPlayerStatus, createAudioPlayer, VoiceConnection} from "@discordjs/voice";
import createVoiceConnection from "../utils/createVoiceConnection";
import createAudioSource from "../utils/createAudioSource";
import Queue from "./Queue";

export default class Session {

    private readonly player: AudioPlayer;
    private readonly connection: VoiceConnection;

    private queue: Queue<Song> = new Queue();
    private current: Song | null = null;

    constructor(guildChannel: GuildChannel) {

        this.player = createAudioPlayer();
        this.connection = createVoiceConnection(guildChannel);
        this.connection.subscribe(this.player);

        this.player.on('error', console.error);
        this.connection.on('error', console.error);
        this.connection.on('stateChange', (oldState, newState) => console.log("Connection", `${oldState.status}->${newState.status}`));
        
        this.player.on('stateChange', (oldState, newState) => console.log("Player", `${oldState.status}->${newState.status}`));

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
        this.stop();
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