import {interactionPayload} from '../types';
import {Client as DiscordClient} from "discord.js";
import youtubeDecoder from "ytdl-core";
import error from "../errors";

export default class SlashCommandService {

    async handle(interaction: interactionPayload, client: DiscordClient) {

        const {member, guild_id, data, channel_id} = interaction;

        const channel = await client.channels.cache.get(channel_id);
        const guild = await client.guilds.fetch(guild_id);
        const caller = await guild.members.fetch(member.user.id);

        const { value: url } = (data.options as any[]).find(o => o.name === 'url');

        if (!guild || !caller) return;

        if (!caller.voice.channel) {
            if(channel && channel.isText()) {
                return channel.send(error('NOT_IN_VOICE_CHANNEL', caller.id));
            }
            return;
        }

        const connection = await caller.voice.channel.join();
        connection.play(youtubeDecoder(url,{ filter: 'audioonly' }));
    }
}