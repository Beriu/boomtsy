import { joinVoiceChannel } from "@discordjs/voice";
import {GuildChannel} from "discord.js";

export default function createVoiceConnection(channel: GuildChannel) {
    return joinVoiceChannel({
        channelId: channel.id,
        guildId: channel.guild.id,
        adapterCreator: channel.guild.voiceAdapterCreator,
    });
}