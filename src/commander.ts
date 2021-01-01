import axios from "axios";
import SlashCommandRepository from "./repositories/SlashCommandRepository";
import * as path from 'path';

const { DISCORD_APP_ID, TEST_GUILD_ID, DISCORD_BOT_TOKEN } = process.env;

(async function() {
    const [executable, file, action, metadata, environment] = process.argv;

    const productionUrl = `https://discord.com/api/v8/applications/${DISCORD_APP_ID}`;
    const developmentUrl = `https://discord.com/api/v8/applications/${DISCORD_APP_ID}/guilds/${TEST_GUILD_ID}`;

    const baseUrl = environment === 'production' ? productionUrl : developmentUrl;
    const instance = axios.create({
        baseURL: baseUrl,
        headers: {
            'Authorization': `Bot ${DISCORD_BOT_TOKEN}`,
            'Content-Type': 'application/json',
        }
    });
    const repo = new SlashCommandRepository(instance);

    switch (action) {
        case 'read':
            repo.read().then(cmds => console.table(cmds));
            break;
        case 'create':
            const resolvedPath = path.resolve(metadata);
            const { default: cmd } = await import(resolvedPath);
            repo.create(cmd).then(cmd => console.log(cmd));
            break
        case 'delete':
            repo.delete(metadata).then(r => console.log(r));
            break;
        default:
             console.error('Invalid action argument.');
             break;
    }
})();

