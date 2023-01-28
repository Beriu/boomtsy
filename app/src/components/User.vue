<script setup lang="ts">
defineProps<{ user: any }>();

const avatarUrl = (userId: string, avatarId: string) => `https://cdn.discordapp.com/avatars/${userId}/${avatarId}.jpg`;
</script>

<template>
    <section v-if="user.id && user.avatar">
        <VAvatar class="mr-4" :image="avatarUrl(user.id, user.avatar)" size="50" />
        <span>{{ user.username }}</span>
    </section>
    <p v-else> loading ...</p>
</template>

<style scoped>
    section {
        display: flex;
        align-items: center;
    }
</style>