import {Snowflake} from "discord.js";

export class NotInVoiceChannelException extends Error {

    constructor(userId: Snowflake) {
        super(`Silly <@${userId}>, you need to join a voice channel first!`);
    }
}

export class InvalidYoutubeUrlException extends Error {

    constructor(userId: Snowflake) {
        super(`<@${userId}> that's not a valid youtube url.`);
    }
}

export class NotTextChannelException extends Error {

    constructor() {
        super('Command was run in an abnormal environment.');
    }
}

export class CommandOutsideGuildException extends Error {

    constructor() {
        super('Cannot run a command outside a guild.');
    }
}