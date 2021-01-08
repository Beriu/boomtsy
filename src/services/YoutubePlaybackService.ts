import youtubeDecoder from 'ytdl-core';
import youtubeSearch from 'ytsr';
import {userInteractionRequest} from "../types";
import NewSongMessage from "../embeds/NewSongMessage";

export default class YoutubePlaybackService {

    constructor() {}

    protected static isValidUrl(url: string): boolean {
        return /((http(s)?:\/\/)?)(www\.)?((youtube\.com\/)|(youtu.be\/))[\S]+/g.test(url);
    }

    async handle({data, member}: userInteractionRequest) {

        const {options: [{value: url}]} = data;

        if (!YoutubePlaybackService.isValidUrl(url)) {

            let {items} = await youtubeSearch(url, {limit: 5});
            items = items.filter(i => i.type === "video");
            const [firstVideo] = items;

            const {user} = member;
            const {title, url: videoUrl, bestThumbnail, duration} = firstVideo as any;
            const userAvatar = user.avatarURL() ?? user.defaultAvatarURL;

            return new NewSongMessage(
                {title, duration, url: videoUrl, thumbnail: bestThumbnail.url},
                {name: user.username, avatar: userAvatar}
            );
        }
    }
}