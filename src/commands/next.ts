import {SlashCommandBuilder} from "@discordjs/builders";
import {Collection, CommandInteraction} from "discord.js";
import {CommandOutsideGuildException} from "../errors";
import Session from "../services/Session";


export default {

    data: new SlashCommandBuilder()
        .setName('next')
        .setDescription('Plays next song from queue.'),

    async execute(interaction: CommandInteraction, sessions: Collection<string, Session>) {
        
        if(!interaction.guild) throw new CommandOutsideGuildException();
        
        const session = sessions.get(interaction.guild.id);
        if(!session) return interaction.editReply({ content: 'No stream is running.' });
        
        const nextSong = await session.next();

        if(!nextSong) {
            return interaction.editReply({ content: 'No new song after current.' });
        }
        interaction.editReply({ content: nextSong.title });
    },
};
