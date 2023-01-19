<script setup lang="ts">
import { onMounted } from "vue";
import state from "./state";
import useAuth from "./useAuth";

const clientId = "794506117497618452";
const redirectUri = encodeURIComponent(window.location.origin);
const loginLink = `https://discord.com/api/oauth2/authorize?client_id=${clientId}&redirect_uri=${redirectUri}&response_type=code&scope=identify`;

const { getToken, setToken } = useAuth();

const extractCodeFromUrl = (url: string) => {
    // window.location.hash
    const fragment = new URLSearchParams(url.slice(1));
    return fragment.get("code");
};

const fetchUser = async ({ accessToken, tokenType }) => {
    const raw = await fetch('https://discord.com/api/users/@me', {
        headers: { 
            authorization: `${tokenType} ${accessToken}`
        },
    });
    const user = await raw.json();
    return user;
};

onMounted(async () => {
    const code = extractCodeFromUrl(window.location.search);

    if(code) {
        const response = await fetch('/api/auth', {
            method: 'POST',
            body: JSON.stringify({ code }),
            headers: { 'Content-Type': 'application/json' }
        });
        const { accessToken, expiresIn } = await response.json();
        setToken(accessToken, expiresIn);
        window.history.replaceState({}, '', window.location.origin);
    }
});

</script>

<template>
    <main>
        <p v-if="getToken()">You're logged in! {{ state.user }}</p>
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
