import {SlashCommandBuilder} from "@discordjs/builders";
import {Collection, CommandInteraction} from "discord.js";
import {CommandOutsideGuildException} from "../errors";
import Session from "../services/Session";


export default {

    data: new SlashCommandBuilder()
        .setName('list')
        .setDescription('Show queued songs.'),

    async execute(interaction: CommandInteraction, sessions: Collection<string, Session>) {

        if(!interaction.guild) throw new CommandOutsideGuildException();
        const session = sessions.get(interaction.guild.id);

        if(!session) return await interaction.reply({ content: 'No stream is running.' });

        const { current, queue } = session.songs();
        const queuedSongs = queue.map(s => `${s.title}`).join("\n");
        const currentSong = current ? `-> ${current.title}` : "";

        return await interaction.reply({ content: `${currentSong} \n ${queuedSongs}` });
    },
};
