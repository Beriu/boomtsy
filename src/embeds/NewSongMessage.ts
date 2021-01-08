import { MessageEmbed } from "discord.js";

export default class NewSongMessage extends MessageEmbed {

    constructor(song: { title: string, thumbnail: string, url: string, duration: string }, user: { avatar: string, name: string }) {
        super(
            {
                color: '#7DE2D1',
                author: { iconURL: user.avatar, name: `${user.name} queued a new song.` },
                title: song.title,
                url: song.url,
                thumbnail: { url: song.thumbnail },
                fields: [
                    { name: 'Duration', value: song.duration, inline: true },
                    { name: 'Position', value: 2, inline: true },
                ]
            }
        );
        this.setTimestamp();
    }
}