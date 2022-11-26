import {REST} from "@discordjs/rest";
import {Routes} from "discord-api-types/v9";
import {Collection} from "discord.js";

export default class CommandsRepository {

    private client: REST;

    constructor(private appId: string, private botToken: string) {
        this.client = new REST({ version: '9' }).setToken(this.botToken);
    }

    updateCommands = (guildId: string, commands: Collection<any, any>) => {
        const body = Array.from(commands).map(([key, cmd]) => cmd.data.toJSON());
        return this.client.put(
            Routes.applicationGuildCommands(this.appId, guildId),
            { body },
        );
    }
}