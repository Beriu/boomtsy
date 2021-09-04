import {SlashCommandBuilder} from "@discordjs/builders";
import {Collection, CommandInteraction} from "discord.js";
import {Song} from "../types";
import {CommandOutsideGuildException} from "../errors";


export default {

    data: new SlashCommandBuilder()
        .setName('stop')
        .setDescription('Stops stream and clears queue.'),

    async execute(interaction: CommandInteraction, queues: Collection<string, Array<Song>>) {
        if(!interaction.guild) throw new CommandOutsideGuildException();
        if(!queues.get(interaction.guild.id)) await interaction.reply({ content: 'No stream is running.' });
        queues.delete(interaction.guild.id);
        await interaction.reply({ content: 'Stop playing and cleared queue.' });
    },
};
