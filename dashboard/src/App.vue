<script setup lang="ts">
import { ref } from 'vue';

const counter = ref(0);

const add = () => counter.value++;
</script>

<template>
    <main>
        <div>
            {{ counter  }}
            <button @click="add()">add</button>
        </div>
    </main>
</template>

<style scoped>
    main {
        display: grid;
        place-items: center;
        height: 100vh;
    }
</style>
