import {SlashCommandBuilder, SlashCommandStringOption} from "@discordjs/builders";
import {Collection, CommandInteraction, GuildMember} from "discord.js";
import YoutubeVideoInfoService from "../services/YoutubeVideoInfoService";
import {Song} from "../types";
import {CommandOutsideGuildException} from "../errors";
import createVoiceConnection from "../utils/createVoiceConnection";
import ytdl from "ytdl-core";
import {createAudioPlayer, createAudioResource, generateDependencyReport} from "@discordjs/voice";


export default {

    data: new SlashCommandBuilder()
        .setName('play')
        .setDescription('Stream the audio of a youtube video.')
        .addStringOption(
            new SlashCommandStringOption()
            .setName('input')
            .setDescription('Youtube video or keywords like song title and/or author.')
            .setRequired(true)
        ),

    async execute(interaction: CommandInteraction, queues: Collection<string, Array<Song>>) {

        let guildQueue: Array<Song>;

        if(!interaction.guild) throw new CommandOutsideGuildException();

        if(!queues.get(interaction.guild.id)) {
            queues.set(interaction.guild.id, []);
        }
        guildQueue = queues.get(interaction.guild.id) as Array<Song>;

        await interaction.deferReply({ ephemeral: true });

        const song = await YoutubeVideoInfoService.search(interaction.options.getString('input') as string)

        guildQueue.push(song);

        await interaction.editReply({ content: song.title });

        const voiceChannel = (interaction.member as GuildMember)?.voice.channel;

        if(voiceChannel) {
            const connection = createVoiceConnection(voiceChannel);
            const player = createAudioPlayer();
            player.play(createAudioResource(ytdl(song.url, { filter: 'audioonly' })));
            connection.subscribe(player);
        }
    },
};
