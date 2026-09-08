<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import type { Component } from 'vue'

defineProps<{
	icon: Component
	title: string
	description: string
	selected?: boolean
	disabled?: boolean
	value: string
}>()

defineEmits<{
	select: [value: string]
}>()
</script>

<template>
	<button
		class="group flex flex-col rounded-xl border-2 p-4 transition-all duration-300 cursor-pointer text-left"
		:class="[
			selected
				? 'border-brand bg-brand-highlight'
				: 'border-surface-4 bg-surface-2 hover:border-surface-5',
			disabled ? 'opacity-60 pointer-events-none' : '',
		]"
		:disabled="disabled"
		@click="$emit('select', value)"
	>
		<div class="flex items-center gap-3">
			<component :is="icon" class="size-6 text-secondary shrink-0" stroke-width="1.5" />
			<div class="flex flex-col min-w-0 flex-1">
				<span class="text-sm font-semibold text-contrast">{{ title }}</span>
				<span class="text-xs text-secondary">{{ description }}</span>
			</div>
			<CheckIcon v-if="selected" class="size-5 text-brand shrink-0" stroke-width="2.5" />
		</div>
		<div
			class="overflow-hidden transition-all duration-300"
			:class="
				selected
					? 'max-h-24 mt-3 opacity-100'
					: 'max-h-0 group-hover:max-h-24 group-hover:mt-3 group-hover:opacity-100 opacity-0'
			"
		>
			<slot />
		</div>
	</button>
</template>
