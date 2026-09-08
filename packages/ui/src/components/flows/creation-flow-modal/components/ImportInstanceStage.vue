<template>
	<div data-onboarding-id="creation-import" class="flex flex-col gap-4">
		<div data-onboarding-id="creation-import-methods" class="flex flex-col gap-3">
			<BigOptionButton
				data-onboarding-id="creation-import-file"
				:icon="FileIcon"
				no-icon-border
				:title="formatMessage(messages.selectFile)"
				:description="formatMessage(messages.selectFileDescription)"
				@click="handleOpenFilePicker"
			/>
			<BigOptionButton
				data-onboarding-id="creation-import-folder"
				:icon="FolderIcon"
				no-icon-border
				:title="formatMessage(messages.selectFolder)"
				:description="formatMessage(messages.selectFolderDescription)"
				@click="handleOpenFolderPicker"
			/>
		</div>

		<span class="text-sm text-secondary">
			{{ formatMessage(messages.importPrompt) }}
		</span>
	</div>
</template>

<script setup lang="ts">
import { FileIcon, FolderIcon } from '@modrinth/assets'
import { BigOptionButton, defineMessages, useVIntl } from '@modrinth/ui'

import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	selectFile: {
		id: 'creation-flow.modal.import-instance.select-file',
		defaultMessage: 'Select file to import',
	},
	selectFileDescription: {
		id: 'creation-flow.modal.import-instance.select-file.description',
		defaultMessage: 'Import a modpack file or launcher archive',
	},
	selectFolder: {
		id: 'creation-flow.modal.import-instance.select-folder',
		defaultMessage: 'Select folder to import',
	},
	selectFolderDescription: {
		id: 'creation-flow.modal.import-instance.select-folder.description',
		defaultMessage: 'Import a launcher folder or .minecraft folder',
	},
	importPrompt: {
		id: 'creation-flow.modal.import-instance.import-prompt',
		defaultMessage:
			'Drag & drop launcher folders, modpack files, or .minecraft folders to import an instance in one click',
	},
})

// ── Native file picker ──
async function handleOpenFilePicker() {
	try {
		const { open } = await import('@tauri-apps/plugin-dialog')
		const result = await open({
			multiple: false,
		})
		const filePath = typeof result === 'string' ? result : (result?.path ?? null)
		if (!filePath) return

		if (ctx.onImportFileReceived) {
			ctx.onImportFileReceived({
				file: null,
				filePath,
				source: 'file-picker',
			})
			return
		}

		// Fallback: set path directly on context
		ctx.modpackFile.value = null
		ctx.modpackFilePath.value = filePath
		if (ctx.finishDisabled.value) return
		if (ctx.flowType === 'instance') {
			ctx.finish()
		} else {
			ctx.modal.value?.setStage('final-config')
		}
	} catch {
		// do nothing
	}
}

// ── Native folder picker ──
async function handleOpenFolderPicker() {
	try {
		const { open } = await import('@tauri-apps/plugin-dialog')
		const result = await open({ multiple: false, directory: true })
		const filePath = typeof result === 'string' ? result : (result?.path ?? null)
		if (!filePath) return

		if (ctx.onImportFileReceived) {
			ctx.onImportFileReceived({
				file: null,
				filePath,
				source: 'file-picker',
			})
			return
		}

		// Fallback: set path directly on context
		ctx.modpackFile.value = null
		ctx.modpackFilePath.value = filePath
		if (ctx.finishDisabled.value) return
		if (ctx.flowType === 'instance') {
			ctx.finish()
		} else {
			ctx.modal.value?.setStage('final-config')
		}
	} catch {
		// do nothing
	}
}
</script>
