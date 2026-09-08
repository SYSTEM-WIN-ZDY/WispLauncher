<template>
	<button
		type="button"
		class="w-full cursor-pointer border-none bg-transparent p-0 text-left"
		:class="[props.disabled ? 'cursor-not-allowed opacity-50' : '']"
		:disabled="props.disabled"
		@click="handleClick"
	>
		<div
			:class="[
				'flex flex-col items-center justify-center border-2 border-dashed bg-surface-4 text-contrast transition-colors',
				size === 'small' ? 'p-5' : size === 'medium' ? 'p-10' : 'p-12',
				size === 'small' ? 'gap-2' : 'gap-4',
				size === 'small' ? 'rounded-2xl' : 'rounded-3xl',
				'border-surface-5',
			]"
		>
			<div
				v-if="!noIconBox"
				:class="[
					'grid place-content-center text-brand border-brand border-solid border bg-highlight-green',
					size === 'small' ? 'w-10 h-10' : 'h-14 w-14',
					size === 'small' ? 'rounded-xl' : 'rounded-2xl',
				]"
			>
				<FolderUpIcon
					aria-hidden="true"
					:class="['text-brand', size === 'small' ? 'w-6 h-6' : 'w-8 h-8']"
				/>
			</div>
			<div v-else class="grid place-content-center">
				<FolderUpIcon
					aria-hidden="true"
					:class="['text-secondary', size === 'small' ? 'w-6 h-6' : 'w-8 h-8']"
				/>
			</div>
			<div class="flex flex-col items-center justify-center gap-1 text-contrast text-center">
				<div class="text-contrast font-medium text-pretty">{{ primaryPrompt }}</div>
				<span v-if="secondaryPrompt" class="text-primary text-sm text-pretty">{{
					secondaryPrompt
				}}</span>
			</div>
		</div>
	</button>
</template>

<script setup lang="ts">
import { FolderUpIcon } from '@modrinth/assets'

import { useDebugLogger } from '#ui/composables/debug-logger'

const debug = useDebugLogger('DropzoneFileInput')

const emit = defineEmits<{
	(e: 'change', paths: string[]): void
}>()

const props = withDefaults(
	defineProps<{
		primaryPrompt?: string | null
		secondaryPrompt?: string | null
		multiple?: boolean
		accept?: string
		disabled?: boolean
		size?: 'small' | 'medium' | 'large'
		directory?: boolean
		noIconBox?: boolean
	}>(),
	{
		primaryPrompt: 'Drop files here or click to upload',
		secondaryPrompt: 'Only supported file types will be accepted',
		size: 'large',
		directory: false,
		noIconBox: false,
	},
)

async function handleClick() {
	debug('handleClick called, disabled:', props.disabled)
	if (props.disabled) {
		debug('disabled, returning')
		return
	}

	try {
		debug('importing @tauri-apps/plugin-dialog')
		const { open } = await import('@tauri-apps/plugin-dialog')
		debug('open function imported')

		if (props.directory) {
			debug('directory mode')
			const result = await open({ directory: true, multiple: false })
			debug('open result:', result)
			const path = typeof result === 'string' ? result : (result?.path ?? null)
			debug('extracted path:', path)
			if (path) {
				debug('emitting change with path:', [path])
				emit('change', [path])
			} else {
				debug('no path selected')
			}
			return
		}

		const filters = props.accept
			? [
					{
						name: props.accept || 'Files',
						extensions: props.accept.split(',').map((ext) => ext.trim().replace(/^\./, '')),
					},
				]
			: undefined
		debug('filters:', filters)

		const result = await open({ multiple: props.multiple ?? false, filters })
		debug('open result:', result)
		const paths = Array.isArray(result) ? result : [result]
		const pickedPaths = paths
			.map((entry) => (typeof entry === 'string' ? entry : entry?.path))
			.filter((p): p is string => !!p)
		debug('pickedPaths:', pickedPaths)

		if (pickedPaths.length > 0) {
			debug('emitting change with pickedPaths:', pickedPaths)
			emit('change', pickedPaths)
		} else {
			debug('no valid paths selected')
		}
	} catch (err) {
		debug('error in handleClick:', err)
		// do nothing
	}
}
</script>
