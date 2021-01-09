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
    Pong = 1, // ACK a Ping
    Acknowledge = 2, // ACK a command without sending a message, eating the user's input
    ChannelMessage = 3, // respond with a message, eating the user's input
    ChannelMessageWithSource = 4, // respond with a message, showing the user's input
    AcknowledgeWithSource = 5, // ACK a command without sending a message, showing the user's input
}

/** Temporary type until Discord updates package typings **/

export type userInteractionOption = { name: string, value: string };

export type userInteraction = {
    id: Snowflake;
    name: string;
    options: Array<userInteractionOption>;
};

export type userInteractionRequest = {
    id: Snowflake;
    guild_id: Snowflake;
    channel_id: Snowflake;
    type: interactionType;
    data: userInteraction;
    member: GuildMember;
    token: string;
    readonly version: 1
};

export type song = { title: string, url: string, duration: string, thumbnail: string };