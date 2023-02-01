import {SlashCommandBuilder} from "@discordjs/builders";
import {CommandOutsideGuildException} from "../errors";
import { commandHandler } from "../utils/loadCommands";

const execute: commandHandler = async ({ interaction, sessions, bridge }) => {
    if(!interaction.guild) throw new CommandOutsideGuildException();
        
    const session = sessions.get(interaction.guild.id);
    if(!session) return interaction.editReply({ content: 'No stream is running.' });
    
    const nextSong = await session.next();

    if(!nextSong) {
        return interaction.editReply({ content: 'No new song after current.' });
    }
    bridge.emit('playlist/refresh', session.songs());
    interaction.editReply({ content: nextSong.title });
};

export default {

    data: new SlashCommandBuilder()
        .setName('next')
        .setDescription('Plays next song from queue.'),

    execute,
};
