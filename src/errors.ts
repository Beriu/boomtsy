
const errors = {
    NOT_IN_VOICE_CHANNEL: (userId: any) => `Silly <@${userId}>, you need to join a voice channel first!`
};

type errorKeys = keyof typeof errors;

export default function error(err: errorKeys, tag?: any) {
    return errors[err](tag);
}