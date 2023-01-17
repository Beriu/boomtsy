import { reactive } from "vue";

export interface AuthCredentials {
    accessToken: string,
    tokenType:   string,
    expiresIn:   number,
};

const state: { auth: AuthCredentials, user: {} } = {
    auth: {
        accessToken: "",
        tokenType:   "",
        expiresIn:   0,
    },
    user: {},
};

export default reactive(state);