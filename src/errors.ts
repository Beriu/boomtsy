
const errors = {
    NOT_IN_VOICE_CHANNEL: (userId: any) => `Silly <@${userId}>, you need to join a voice channel first!`,
    INVALID_YOUTUBE_URL: (userId: any) => `<@${userId}> that's not a valid youtube url.`
};

type errorKeys = keyof typeof errors;

const error = (err: errorKeys, tag?: any) => errors[err](tag);

export default error;