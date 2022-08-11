import {ActivityType, Client, Collection, GatewayIntentBits} from "discord.js";
import CommandsRepository from "./repositories/CommandsRepository";
import loadCommands from "./utils/loadCommands";
import Session from "./services/Session";

async function run() {

    const { DISCORD_BOT_TOKEN, DISCORD_APP_ID, TEST_GUILD_ID } = process.env as { [key: string]: any };

    const client = new Client({
        presence: {
            activities: [{ name: 'BANGERS 🔥', type: ActivityType.Listening }]
        },
        intents: [
            GatewayIntentBits.Guilds,
            GatewayIntentBits.GuildVoiceStates,
        ]
    });

    const commands = loadCommands(__dirname + '/commands');
    await (new CommandsRepository(DISCORD_APP_ID, DISCORD_BOT_TOKEN)).updateCommands(TEST_GUILD_ID, commands);

    const sessions: Collection<string, Session> = new Collection();

    client.once('ready', () => console.info('Bot is running...'));

    client.on('interactionCreate', async interaction => {

        if (!interaction.isCommand()) return;
        const command = commands.get(interaction.commandName);
        if (!command) return;

        try {
            await command.execute(interaction, sessions);
        } catch (error) {
            await interaction.editReply({ content: (error as Error).message });
        }
    });

    await client.login(DISCORD_BOT_TOKEN);
}

void run();




