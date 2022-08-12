import { joinVoiceChannel } from "@discordjs/voice";
import {GuildChannel} from "discord.js";

export default function createVoiceConnection(channel: GuildChannel) {
    console.log('Trying to create new voice connection:', channel.guild.id);
    return joinVoiceChannel({
        channelId: channel.id,
        guildId: channel.guild.id,
        adapterCreator: channel.guild.voiceAdapterCreator,
    });
}