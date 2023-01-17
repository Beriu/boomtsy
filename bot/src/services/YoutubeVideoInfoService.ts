import ytdl from 'ytdl-core';
import ytsr, {Video} from 'ytsr';
import { Song } from "../types";

export default class YoutubeVideoInfoService {

    constructor() {}

    protected static isValidUrl(url: string): boolean {
        return /((http(s)?:\/\/)?)(www\.)?((youtube\.com\/)|(youtu.be\/))[\S]+/g.test(url);
    }

    protected static async youtubeSearchByQuery(params: string): Promise<Song> {
        let {items} = await ytsr(params, {limit: 5, });
        items = items.filter(i => i.type === "video" && !i.isLive);
        const [firstVideo] = items as Array<Video>;
        const { title, url: videoUrl, bestThumbnail, duration } = firstVideo;
        return {title, duration, url: videoUrl, thumbnail: bestThumbnail.url} as Song;
    }

    protected static async youtubeSearchByUrl(url: string): Promise<Song> {
        const { videoDetails: { title, lengthSeconds, thumbnails: [firstThumbnail], video_url } }  = await ytdl.getBasicInfo(url);
        const duration = YoutubeVideoInfoService.convertSecondsToStdFormat(parseInt(lengthSeconds));
        return {title, duration, url: video_url, thumbnail: firstThumbnail.url};
    }

    protected static convertSecondsToStdFormat(seconds: number): string {
        const minutes = ~~(seconds / 60);
        let leftoverSeconds: string | number = seconds % 60;
        leftoverSeconds = leftoverSeconds.toString().length < 2 ? `0${leftoverSeconds}` : `${leftoverSeconds}`;
        return `${minutes}:${leftoverSeconds}`;
    }

    public static async search(input: string) {
        return YoutubeVideoInfoService.isValidUrl(input)
            ? await YoutubeVideoInfoService.youtubeSearchByUrl(input)
            : await YoutubeVideoInfoService.youtubeSearchByQuery(input);
    }
}