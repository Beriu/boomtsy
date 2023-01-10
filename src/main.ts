import {ActivityType, Client, Collection, GatewayIntentBits, PresenceUpdateStatus} from "discord.js";
import CommandsRepository from "./repositories/CommandsRepository";
import loadCommands from "./utils/loadCommands";
import Session from "./services/Session";
import express from "express";
import { Server } from "socket.io";
import http from "http";
import path from "path";

const sessions: Collection<string, Session> = new Collection();

async function runBot() {

    const { DISCORD_BOT_TOKEN, DISCORD_APP_ID, TEST_GUILD_ID } = process.env as Record<string, any>;

    const client = new Client({
        presence: {
            activities: [{ name: 'BANGERS 🔥', type: ActivityType.Listening }],
            status: PresenceUpdateStatus.Online,
        },
        intents: [
            GatewayIntentBits.Guilds,
            GatewayIntentBits.GuildVoiceStates,
            GatewayIntentBits.GuildMessages
        ]
    });

    const repo = new CommandsRepository(DISCORD_APP_ID, DISCORD_BOT_TOKEN);
    const commands = loadCommands(__dirname + '/commands');
    await repo.updateCommands(TEST_GUILD_ID, commands);

    client.once('ready', () => console.info('Bot is running...'));
    client.on('error', console.info);

    client.on('interactionCreate', async interaction => {

        if (!interaction.isCommand()) return;

        await interaction.deferReply();

        const command = commands.get(interaction.commandName);

        if (!command) {
            await interaction.editReply({ content: `${interaction.commandName} is not a valid command.` })
            return;
        }

        try {
            await command.execute(interaction, sessions);
        } catch (error) {
            await interaction.editReply({ content: (error as Error).message });
        }
    });

    await client.login(DISCORD_BOT_TOKEN);
}

async function runServer() {
    const app = express();
    const server = http.createServer(app);
    const socket = new Server(server);

    app.use('', express.static(path.join(__dirname, '../dashboard')));

    socket.on('connection', (socket) => {
        socket.emit("sessions/get", 'Hi there! This is on CibzPi');
    });

    server.listen(process.env.WEBSERVER_PORT, () => {
        console.log(`Webserver running at http://localhost:${process.env.WEBSERVER_PORT}`);
    });
}

void runBot();
void runServer();




