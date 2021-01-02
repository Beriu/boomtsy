import {interactionPayload} from '../types';
import {Client as DiscordClient} from "discord.js";
import youtubeDecoder from "ytdl-core";

export default class SlashCommandService {

    async handle(interaction: interactionPayload, client: DiscordClient) {

        const {member, guild_id, data} = interaction;

        const guild = await client.guilds.fetch(guild_id);
        const caller = await guild.members.fetch(member.user.id);

        const { value: url } = (data.options as any[]).find(o => o.name === 'url');
        
        if (!guild || !caller) return;

        if (caller.voice.channel) {
            const connection = await caller.voice.channel.join();
            connection.play(youtubeDecoder(url,{ filter: 'audioonly' }));
        } else {
            const textChannel = client.channels.cache.get(interaction.channel_id);
            if(textChannel && textChannel.isText())
                return textChannel.send('You need to join a voice channel first!');
        }
    }
}