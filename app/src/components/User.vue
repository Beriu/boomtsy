<script setup lang="ts">
defineProps<{ user: any, isConnected: boolean }>();

const avatarUrl = (userId: string, avatarId: string) => `https://cdn.discordapp.com/avatars/${userId}/${avatarId}.jpg`;
</script>

<template>
    <section v-if="user.id && user.avatar">
        <v-avatar class="mr-4" :image="avatarUrl(user.id, user.avatar)" size="50" />
        
        <div class="d-inline-flex justify-space-between w-100 align-center">
            <span class="text-title">{{ user.username }}</span>
            <span class="text-caption">
                <v-chip
                    class="ma-2"
                    :color="isConnected ? 'success' : 'error'"
                    variant="outlined">
                    <v-icon start icon="mdi-account"></v-icon>
                    {{ isConnected ? 'Online' : 'Offline' }}
                </v-chip>
            </span>
        </div>
    </section>

    <p v-else> loading ...</p>
</template>

<style scoped>
    section {
        display: flex;
        align-items: center;
    }
</style>