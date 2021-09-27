import YoutubeVideoInfoService from "../src/services/YoutubeVideoInfoService";
import ytsr from 'ytsr';

test('Youtube search test.', async () => {

    const response = await ytsr('oof', { limit: 5, safeSearch: false });

    console.log(response);
});