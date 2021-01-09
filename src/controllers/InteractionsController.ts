import {Client as DiscordClient, GuildMember, Snowflake, TextChannel} from "discord.js";
import {userInteractionRequest} from "../types";
import error from "../errors";
import YoutubeVideoInfoService from "../services/YoutubeVideoInfoService";
import NewSongMessage from "../embeds/NewSongMessage";
import ytdl from "ytdl-core";

export default class InteractionsController {

    constructor(protected client: DiscordClient) {}

    protected async getTextChannel(channelId: Snowflake): Promise<TextChannel | false> {
        let textChannel = this.client.channels.cache.get(channelId);
        if(!textChannel) textChannel = await this.client.channels.fetch(channelId);
        if(!textChannel.isText()) return false;
        if(!(textChannel instanceof TextChannel)) return false;
        return textChannel;
    }

    protected static getUsernameAndAvatar({ user }: GuildMember) {
        return {
            name: user.username,
            avatar: user.avatarURL() ?? user.defaultAvatarURL
        }
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

        const song = await YoutubeVideoInfoService.handle(interaction);
        const embed = new NewSongMessage(song, InteractionsController.getUsernameAndAvatar(guildMember));

        await textChannel.send([embed]);

        const connection = await guildMember.voice.channel.join();
        const isDeaf = await connection.voice?.setSelfDeaf(true);
        const stream = ytdl(song.url, { filter: 'audioonly' });

        connection.play(stream);
    }
}