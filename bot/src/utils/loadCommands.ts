import {Collection} from "discord.js";
import fs from "fs";
import {SlashCommandBuilder} from "@discordjs/builders";

type commandWrapper = { data: SlashCommandBuilder, execute: CallableFunction };

export default function loadCommands(path: string) {

    const commands: Collection<string, commandWrapper> = new Collection();
    const commandFiles = fs.readdirSync(path);

    for (const file of commandFiles) {
        const {default: command} = require(`${path}/${file}`);
        commands.set(command.data.name, command);
    }

    return commands;
}