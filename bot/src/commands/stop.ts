import {SlashCommandBuilder} from "@discordjs/builders";
import {Collection, CommandInteraction} from "discord.js";
import {CommandOutsideGuildException} from "../errors";
import Session from "../services/Session";


export default {

    data: new SlashCommandBuilder()
        .setName('stop')
        .setDescription('Stops stream and clears queue.'),

    async execute(interaction: CommandInteraction, sessions: Collection<string, Session>) {
        if(!interaction.guild) throw new CommandOutsideGuildException();
        const session = sessions.get(interaction.guild.id);
        if(!session) return await interaction.editReply({ content: 'No stream is running.' });
        session.stop();
        sessions.delete(interaction.guild.id);
        await interaction.editReply({ content: 'Stop playing and cleared queue.' });
    },
};
