import {Client as DiscordClient, Message, WSEventType } from "discord.js";
import {interactionPayload} from "../types";

export default class App {

    protected client: DiscordClient

    constructor() {
        this.client = new DiscordClient();
    }

    protected onMessageHandler = async (msg: Message) => {
        switch (msg.content) {
            case 'ping':
                return msg.reply('pong');
        }
    }

    protected onReadyHandler = async () => {
        console.info(`${this.client.user?.tag} started listening.`);
    }

    protected onInteractionHandler = async (payload: interactionPayload) => {
        console.log({ payload });
    }

    async start() {
        const { DISCORD_BOT_TOKEN } = process.env;

        /** For normal message events in text channels **/
        this.client.on('ready', this.onReadyHandler);
        this.client.on('message', this.onMessageHandler);

        /** For new feature with slash commands **/
        this.client.ws.on('INTERACTION_CREATE' as WSEventType, this.onInteractionHandler);

        await this.client.login(DISCORD_BOT_TOKEN);
    }
}