import {Collection, CommandInteraction} from "discord.js";
import fs from "fs";
import {SlashCommandBuilder} from "@discordjs/builders";
import Session from "../services/Session";
import EventEmitter from "node:events";

export type commandHandlerParams = { 
    interaction: CommandInteraction,
    sessions: Collection<string, Session>,
    bridge: EventEmitter
};

export type commandHandler = (params: commandHandlerParams) => void;

type commandWrapper = { data: SlashCommandBuilder, execute: commandHandler };

export default function loadCommands(path: string) {

    const commands: Collection<string, commandWrapper> = new Collection();
    const commandFiles = fs.readdirSync(path);

    for (const file of commandFiles) {
        const {default: command} = require(`${path}/${file}`);
        commands.set(command.data.name, command);
    }

    return commands;
}