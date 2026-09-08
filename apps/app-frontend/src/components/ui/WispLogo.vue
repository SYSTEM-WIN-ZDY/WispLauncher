<template>
	<div aria-label="Wisp" class="flex h-full items-center gap-2 font-extrabold text-contrast">
		<img aria-hidden="true" class="aspect-square h-full object-contain" :src="wispVisual" />
		<span v-if="!iconOnly" class="hidden text-sm tracking-wide xl:inline">Wisp</span>
	</div>
</template>

<script setup lang="ts">
import wispVisual from '@modrinth/assets/branding/allay.gif'

defineProps<{
	iconOnly?: boolean
}>()
</script>
