<script setup lang="ts">
import mascotVisual from '@/assets/allay.gif'

defineProps<{
	alt: string
}>()
</script>

<template>
	<img class="onboarding-mascot" :src="mascotVisual" :alt="alt" />
</template>

<style scoped lang="scss">
.onboarding-mascot {
	width: clamp(6rem, 11vw, 9rem);
	height: auto;
	object-fit: contain;
}
</style>
