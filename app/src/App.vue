<script setup lang="ts">
import { onMounted } from "vue";
import state from "./state";
import useAuth from "./useAuth";
import User from "./components/User.vue";

const clientId = "794506117497618452";
const redirectUri = encodeURIComponent(window.location.origin);
const loginLink = `https://discord.com/api/oauth2/authorize?client_id=${clientId}&redirect_uri=${redirectUri}&response_type=code&scope=identify`;

const { token, setToken } = useAuth();

const extractCodeFromUrl = (url: string) => {
    // window.location.hash
    const fragment = new URLSearchParams(url.slice(1));
    return fragment.get("code");
};

onMounted(async () => {
    const code = extractCodeFromUrl(window.location.search);
    if(code) {
        try {
            const response = await fetch('/api/auth', {
                method: 'POST',
                body: JSON.stringify({ code, redirectUri: window.location.origin }),
                headers: { 'Content-Type': 'application/json' }
            });
            const { accessToken, expiresIn } = await response.json();
            setToken(accessToken, expiresIn);
            window.history.replaceState({}, '', window.location.origin);
        }catch(error: any) {
            console.error(error);
        }
    }

    if(token) {
        const userResponse = await fetch('/api/user', 
            { headers: { 'Authorization': token.value } }
        );
        const user = await userResponse.json();
        state.user = user;
    }
});

</script>

<template>
    <main>
        <div v-if="token">
            <User :user="state.user" />
        </div>
        <VBtn v-else :href="loginLink" color="#7289DA" class="text-white">
            <VIcon icon="mdi-discord" class="mr-2" color="white"/>
            Log in with Discord
        </VBtn>
    </main>
</template>

<style scoped>
    main {
        display: grid;
        place-items: center;
        height: 100vh;
    }
</style>
