import {SlashCommandBuilder, SlashCommandStringOption} from "@discordjs/builders";
import {Collection, CommandInteraction, GuildMember, VoiceChannel} from "discord.js";
import YoutubeVideoInfoService from "../services/YoutubeVideoInfoService";
import {Song} from "../types";
import {CommandOutsideGuildException} from "../errors";
import Session from "../services/Session";


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

    async execute(interaction: CommandInteraction, sessions: Collection<string, Session>) {

        if(!interaction.guild) throw new CommandOutsideGuildException();

        let session = sessions.get(interaction.guild.id)

        if(!session) {
            sessions.set(interaction.guild.id, new Session());
            session = sessions.get(interaction.guild.id) as Session;
        }

        await interaction.deferReply({ ephemeral: true });

        const song = await YoutubeVideoInfoService.search(interaction.options.getString('input') as string)

        session.addSong(song);

        await interaction.editReply({ content: song.title });

        const voiceChannel = (interaction.member as GuildMember)?.voice.channel as VoiceChannel;

        if(voiceChannel && session.isNotPlaying()) {
            session.start(voiceChannel);
        }
    },
};
