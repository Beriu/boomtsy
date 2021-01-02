import axios from "axios";
import SlashCommandRepository from "./repositories/SlashCommandRepository";
import * as path from 'path';
import {promises as fs} from "fs";
import {slashCommand} from "./types";

const { DISCORD_APP_ID, TEST_GUILD_ID, DISCORD_BOT_TOKEN } = process.env;

async function loadCommands() {
    const cmdMap: slashCommand[] = [];
    const normalizedPath = path.join(__dirname, 'commands');
    const files = await fs.readdir(normalizedPath);

    for(const file of files) {
        const filePath = path.join(__dirname, 'commands/' + file);
        const { default: defaultImport } = await import(filePath);
        cmdMap.push(defaultImport);
    }
    return cmdMap;
}

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

