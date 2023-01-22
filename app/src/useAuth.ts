import Cookie from "js-cookie";
import { ref } from "vue";

export default function useAuth() {
    
    const existingToken = Cookie.get('accessToken');

    const token = ref(existingToken ?? "");
  
    const setToken = (newToken: string, expiresIn: number) => {
        token.value = newToken;
        Cookie.set('accessToken', `Bearer ${newToken}`, { expires: expiresIn / 60 / 60 / 24 });
    }

    return { token, setToken };
};
