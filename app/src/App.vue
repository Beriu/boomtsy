<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import state from "./state";
import useAuth from "./useAuth";
import User from "./components/User.vue";
import type { Song } from "../../bot/src/types";
import { io, Socket } from "socket.io-client";

const clientId = "794506117497618452";
const redirectUri = encodeURIComponent(window.location.origin);
const loginLink = `https://discord.com/api/oauth2/authorize?client_id=${clientId}&redirect_uri=${redirectUri}&response_type=code&scope=identify`;

const { token, setToken } = useAuth();
const songs = ref<Song[]>([]);
const isConnected = ref(false);
let socket: Socket | null = null;

const extractCodeFromUrl = (url: string) => {
    // window.location.hash
    const fragment = new URLSearchParams(url.slice(1));
    return fragment.get("code");
};

const refreshSongs = ({ current, queue }: { current: null | Song, queue: Song[] }) => {
    if(current) {
        songs.value = [ current , ...queue ];
    } else {
        songs.value = queue;
    }
}

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
        const songsFromApi = await connectionsResponse.json();
        refreshSongs(songsFromApi);

        socket = io();
        socket.on('connect', () => isConnected.value = true);
        socket.on('close', () => isConnected.value = false);
        socket.on('sessions/get', console.info);
        socket.on('playlist/refresh', e => refreshSongs(e));
        socket.on('playlist/clear', () => songs.value = []);
    }
});

onUnmounted(() => {
    if(socket) socket.removeAllListeners();
});

</script>

<template>
    <main>
        <div style="width: 400px;" v-if="token">
            <v-card rounded="lg" class="pa-2 mb-2">
                <User :is-connected="isConnected" :user="state.user" />
            </v-card>
            
            <v-card rounded="lg">
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
            </v-card>
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
