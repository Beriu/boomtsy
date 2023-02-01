import { SlashCommandBuilder, SlashCommandStringOption } from "@discordjs/builders";
import {
    GuildMember,
    VoiceChannel
} from "discord.js";
import YoutubeVideoInfoService from "../services/YoutubeVideoInfoService";
import { Song } from "../types";
import { CommandOutsideGuildException, NotInVoiceChannelException } from "../errors";
import Session from "../services/Session";
import { commandHandler } from "../utils/loadCommands";


const execute: commandHandler = async ({ interaction, sessions, bridge }) => {
    if(!interaction.guild) throw new CommandOutsideGuildException();
    if(!interaction.member) throw new CommandOutsideGuildException();

    const voiceChannel = (interaction.member as GuildMember).voice.channel as VoiceChannel;
    if(!voiceChannel) throw new NotInVoiceChannelException(interaction.member.user.id);

    let session = sessions.get(interaction.guild.id);

    if(!session) {
        sessions.set(interaction.guild.id, new Session(voiceChannel));
        session = sessions.get(interaction.guild.id)!;
    }

    const input = interaction.options.get("input")!;
    const song = await YoutubeVideoInfoService.search(input.value as string);

    session.addSong(song);
    bridge.emit('playlist/refresh', session.songs());

    await interaction.editReply({ content: song.title });

    if(session.isNotPlaying()) session.start();
};

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

    execute,
};
