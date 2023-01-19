import Cookie from "js-cookie";
import { ref } from "vue";

export default function useAuth() {
    
    const accessToken = ref("");

    const getToken = () => {
        const existingToken = Cookie.get('accessToken');
        if(existingToken) {
            accessToken.value = existingToken;
            return accessToken;
        }
    }
  
    const setToken = (newToken: string, expiresIn: number) => {
        accessToken.value = newToken;
        Cookie.set('accessToken', `Bearer ${newToken}`, { expires: expiresIn / 60 / 60 / 24 });
    }

    return { getToken, setToken };
};
