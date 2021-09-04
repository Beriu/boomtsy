import { MessageEmbed } from "discord.js";
import { Song } from "../types";

export default class NewSongMessage extends MessageEmbed {

    constructor(song: Song, user: { avatar: string, name: string }, position: number) {
        super(
            {
                color: '#7DE2D1',
                author: { iconURL: user.avatar, name: `${user.name} queued a new song.` },
                title: song.title,
                url: song.url,
                thumbnail: { url: song.thumbnail },
                fields: [
                    { name: 'Duration', value: song.duration as string, inline: true },
                    { name: 'Position', value: position.toString() as string, inline: true },
                ]
            }
        );
        this.setTimestamp();
    }
}