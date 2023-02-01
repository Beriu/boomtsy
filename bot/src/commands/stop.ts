import {SlashCommandBuilder} from "@discordjs/builders";
import {CommandOutsideGuildException} from "../errors";
import { commandHandler } from "../utils/loadCommands";

const execute: commandHandler = async ({ interaction, sessions, bridge }) => {
    if(!interaction.guild) throw new CommandOutsideGuildException();
    const session = sessions.get(interaction.guild.id);
    if(!session) return await interaction.editReply({ content: 'No stream is running.' });
    session.stop();
    sessions.delete(interaction.guild.id);
    bridge.emit('playlist/clear');
    await interaction.editReply({ content: 'Stop playing and cleared queue.' });
};

export default {

    data: new SlashCommandBuilder()
        .setName('stop')
        .setDescription('Stops stream and clears queue.'),

    execute,
};
