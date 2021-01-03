import {interactionPayload} from '../types';
import {Client as DiscordClient, Snowflake, TextChannel} from "discord.js";
import error from "../errors";

export default class SlashCommandService {

    protected async getTextChannel(channelId: Snowflake, client: DiscordClient): Promise<TextChannel | false> {
        let textChannel = client.channels.cache.get(channelId);
        if(!textChannel) textChannel = await client.channels.fetch(channelId);
        if(!textChannel.isText()) return false;
        if(!(textChannel instanceof TextChannel)) return false;
        return textChannel;
    }

    async handle(interaction: interactionPayload, client: DiscordClient) {

        const {member, channel_id} = interaction;

        const textChannel = await this.getTextChannel(channel_id, client);

        if(!textChannel) return;

        const guild = textChannel.guild;
        const caller = await guild.members.fetch(member.user.id);

        if (!guild || !caller) return;

        if (!caller.voice.channel) {
            if(textChannel && textChannel.isText()) {
                return textChannel.send(error('NOT_IN_VOICE_CHANNEL', caller.id));
            }
            return;
        }
        const connection = await caller.voice.channel.join();
        const isDeaf = await connection.voice?.setSelfDeaf(true);

        //connection.play();
    }
}