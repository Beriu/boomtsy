import {GuildMember, Snowflake} from "discord.js";

export type slashCommand = {
    id?: string;
    name: string;
    description: string;
    options: Array<slashCommandOption>;
};

export type slashCommandOption = {
    name: string;
    description: string;
    type: slashCommandOptionType;
    required: boolean;
    choices?: Array<slashCommandOptionChoice>;
};

export type slashCommandOptionChoice = {
    name: string;
    value: string;
};

export enum slashCommandOptionType {
    SUB_COMMAND = 1,
    SUB_COMMAND_GROUP =	2,
    STRING = 3,
    INTEGER = 4,
    BOOLEAN = 5,
    USER = 6,
    CHANNEL = 7,
    ROLE = 8
}

export enum interactionType {
    Ping = 1,
    ApplicationCommand = 2
}

/** Temporary type until Discord updates package typings **/
export type interactionPayload = {
    id: Snowflake;
    guild_id: Snowflake;
    channel_id: Snowflake;
    type: interactionType;
    data: slashCommand;
    member: GuildMember;
    token: string;
    readonly version: 1
};