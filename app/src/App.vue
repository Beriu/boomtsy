<script setup lang="ts">
import { onMounted, ref } from "vue";
import state from "./state";
import useAuth from "./useAuth";
import User from "./components/User.vue";
import type { Song } from "../../bot/src/types";

const clientId = "794506117497618452";
const redirectUri = encodeURIComponent(window.location.origin);
const loginLink = `https://discord.com/api/oauth2/authorize?client_id=${clientId}&redirect_uri=${redirectUri}&response_type=code&scope=identify`;

const { token, setToken } = useAuth();
const songs = ref<Song[]>([]);

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
        } catch(error: any) {
            console.error(error);
        }
    }

    if(token) {
        const userResponse = await fetch('/api/user',
            { headers: { 'Authorization': token.value } }
        );
        const user = await userResponse.json();
        state.user = user;

        const connectionsResponse = await fetch('/api/user/connections',
            { headers: { 'Authorization': token.value } }
        );
        const { error, current, queue } = await connectionsResponse.json();
        if(!error) songs.value = [ current, ...queue ];
    }
});

</script>

<template>
    <main>
        <div v-if="token">
            <User :user="state.user" />

            <v-list v-if="songs.length > 0" lines="two">
                <v-list-subheader>Playlist</v-list-subheader>
            
                <v-list-item 
                    v-for="song in songs" 
                    :key="song.url" 
                    :title="song.title" 
                    :subtitle="song.duration">
                    
                    <template v-slot:prepend>
                        <v-img class="rounded mr-3" width="80" :src="song.thumbnail" />
                    </template>
            
                    <!-- <template v-slot:append>
                        <v-btn color="grey-lighten-1" icon="mdi-information" variant="text"></v-btn>
                    </template> -->
                </v-list-item>
            </v-list>
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
