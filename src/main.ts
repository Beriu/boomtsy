import {ActivityType, Client, Collection, CommandInteraction, GatewayIntentBits, PresenceUpdateStatus} from "discord.js";
import CommandsRepository from "./repositories/CommandsRepository";
import loadCommands from "./utils/loadCommands";
import Session from "./services/Session";
import { generateDependencyReport } from "@discordjs/voice"

console.log(generateDependencyReport());

async function run() {

    const { DISCORD_BOT_TOKEN, DISCORD_APP_ID, TEST_GUILD_ID } = process.env as { [key: string]: any };

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

    const commands = loadCommands(__dirname + '/commands');
    await (new CommandsRepository(DISCORD_APP_ID, DISCORD_BOT_TOKEN)).updateCommands(TEST_GUILD_ID, commands);

    const sessions: Map<string, Session> = new Map();

    client.once('ready', () => console.info('Bot is running...'));

    client.on('interactionCreate', async interaction => {

        if (!interaction.isCommand()) return;
        const command = commands.get(interaction.commandName);
        if (!command) return;

        try {
            await command.execute(interaction, sessions);
        } catch (error) {
            console.log(error);
            await interaction.reply({ content: (error as Error).message });
        }
    });

    await client.login(DISCORD_BOT_TOKEN);
}

void run();




