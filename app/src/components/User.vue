<script setup lang="ts">
defineProps<{ user: any }>();

</script>

<template>
    <section>
        {{ user.username }}#{{ user.discriminator }}
    </section>
</template>