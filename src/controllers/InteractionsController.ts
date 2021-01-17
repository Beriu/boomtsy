import {Client as DiscordClient, Guild, GuildMember, Snowflake, TextChannel, VoiceConnection} from "discord.js";
import {userInteractionRequest} from "../types";
import {NotInVoiceChannelException, NotTextChannelException} from "../errors";
import YoutubeVideoInfoService from "../services/YoutubeVideoInfoService";
import NewSongMessage from "../embeds/NewSongMessage";
import ytdl from "ytdl-core";

type processedInteraction = userInteractionRequest & { channel: TextChannel, guild: Guild, guildMember: GuildMember };

export default class InteractionsController {

    constructor(protected client: DiscordClient) {}

    protected async getTextChannel(channelId: Snowflake): Promise<TextChannel | never> {
        let channel = this.client.channels.cache.get(channelId);
        if(!channel) channel = await this.client.channels.fetch(channelId);
        if(!channel.isText()) throw new NotTextChannelException();
        if(!(channel instanceof TextChannel)) throw new NotTextChannelException();
        return channel;
    }

    protected static getUsernameAndAvatar({ user }: GuildMember) {
        return {
            name: user.username,
            avatar: user.avatarURL() ?? user.defaultAvatarURL
        }
    }

    protected async hydrateInteraction(interaction: userInteractionRequest): Promise<processedInteraction> {
        const {member, channel_id} = interaction;
        const channel = await this.getTextChannel(channel_id);
        const guild = channel.guild;
        const guildMember = await guild.members.fetch(member.user.id);
        return { ...interaction, channel, guild, guildMember };
    }

    protected async validateInteraction(interaction: userInteractionRequest): Promise<processedInteraction> {
        const hydrated = await this.hydrateInteraction(interaction);
        if(!hydrated.guildMember.voice.channel) {
            throw new NotInVoiceChannelException(hydrated.guildMember.user.id);
        }
        //TODO: If there's an active connection on the voice channel,
        // user who ads song must be in same voice channel
        return hydrated;
    }

    onSlashCommand = async (interaction: userInteractionRequest) => {
        try {
            const { channel, guildMember, guild } = await this.validateInteraction(interaction);

            const song = await YoutubeVideoInfoService.handle(interaction);
            const user = InteractionsController.getUsernameAndAvatar(guildMember);
            const embed = new NewSongMessage(song, user, 1);

            await channel.send([embed]);

            const connection = await guildMember.voice.channel?.join() as VoiceConnection;
            await connection.voice?.setSelfDeaf(true);

            const stream = ytdl(song.url, { filter: 'audioonly' });
            const connectionStream = connection.play(stream);

        }catch (error) { console.log(error.message); }
    }
}