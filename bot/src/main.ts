import {ActivityType, Client, Collection, GatewayIntentBits, PresenceUpdateStatus} from "discord.js";
import CommandsRepository from "./repositories/CommandsRepository";
import loadCommands from "./utils/loadCommands";
import Session from "./services/Session";
import EventEmitter from "node:events";

export default async function runBot(sessions: Collection<string, Session>, bridge: EventEmitter) {

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
            await command.execute({ interaction, sessions, bridge });
        } catch (error) {
            await interaction.editReply({ content: (error as Error).message });
        }
    });

    await client.login(DISCORD_BOT_TOKEN);
}






