import {Client as DiscordClient, Snowflake, TextChannel} from "discord.js";
import {userInteractionRequest} from "../types";
import error from "../errors";
import YoutubePlaybackService from "../services/YoutubePlaybackService";

export default class InteractionsController {

    constructor(protected client: DiscordClient) {}

    protected async getTextChannel(channelId: Snowflake): Promise<TextChannel | false> {
        let textChannel = this.client.channels.cache.get(channelId);
        if(!textChannel) textChannel = await this.client.channels.fetch(channelId);
        if(!textChannel.isText()) return false;
        if(!(textChannel instanceof TextChannel)) return false;
        return textChannel;
    }

    onSlashCommand = async (interaction: userInteractionRequest) => {

        const {member, channel_id} = interaction;

        const textChannel = await this.getTextChannel(channel_id);

        if(!textChannel) return;

        const guild = textChannel.guild;
        const guildMember = await guild.members.fetch(member.user.id);
        interaction.member.user = guildMember.user;

        if (!guild || !guildMember) return;

        if (!guildMember.voice.channel) {
            if(textChannel && textChannel.isText()) {
                return textChannel.send(error('NOT_IN_VOICE_CHANNEL', guildMember.id));
            }
            return;
        }

        // const connection = await caller.voice.channel.join();
        // const isDeaf = await connection.voice?.setSelfDeaf(true);

        const yt = new YoutubePlaybackService();
        const embed = await yt.handle(interaction);

        await textChannel.send([embed]);
    }
}