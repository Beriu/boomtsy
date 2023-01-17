<script setup lang="ts">
import { onMounted } from "vue";
import state from "./state";
import type { AuthCredentials } from "./state";

const clientId = "794506117497618452";
const redirectUri = encodeURIComponent(window.location.origin);
const loginLink = `https://discord.com/api/oauth2/authorize?client_id=${clientId}&redirect_uri=${redirectUri}&response_type=token&scope=identify`;

const extractTokensFromUrl = (url: string) => {
    // window.location.hash
    const fragment = new URLSearchParams(url.slice(1));
    const accessToken = fragment.get('access_token') ?? "";
    const tokenType = fragment.get('token_type') ?? "";
    const expiresIn = parseInt(fragment.get('expires_in') ?? "0");
    return { accessToken, tokenType, expiresIn };
};

const fetchUser = async ({ accessToken, tokenType }: AuthCredentials) => {
    const raw = await fetch('https://discord.com/api/users/@me', {
        headers: { 
            authorization: `${tokenType} ${accessToken}`
        },
    });
    const user = await raw.json();
    return user;
};

onMounted(async () => {
    const urlHash = window.location.hash;

    if(urlHash) {
        state.auth = extractTokensFromUrl(urlHash);
        state.user = await fetchUser(state.auth);
        window.location.hash = "";
    }
});

</script>

<template>
    <main>
        <p v-if="state.bearerToken">You're logged in! {{ state.user }}</p>
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
