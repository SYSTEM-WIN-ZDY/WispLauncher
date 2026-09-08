<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { useTemplateRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

const emit = defineEmits<{
	continue: []
	decline: []
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	title: { id: 'app.servers.eula.title', defaultMessage: 'Minecraft EULA' },
	description: {
		id: 'app.servers.eula.description',
		defaultMessage:
			'By continuing, you agree to the Minecraft End User License Agreement (EULA). Please review the agreement below before proceeding.',
	},
	continue: {
		id: 'app.servers.eula.continue',
		defaultMessage: 'Continue',
	},
	decline: { id: 'app.servers.eula.decline', defaultMessage: 'Cancel' },
})

const modal = useTemplateRef<ComponentExposed<typeof NewModal>>('modal')

defineExpose({
	show: (event?: MouseEvent) => modal.value?.show(event),
	hide: () => modal.value?.hide(),
})
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)">
		<div class="flex flex-col gap-4">
			<p class="m-0 text-secondary">
				{{ formatMessage(messages.description) }}
			</p>
		</div>
		<template #actions>
			<div class="flex flex-col justify-end gap-2 sm:flex-row">
				<ButtonStyled type="outlined">
					<button type="button" @click="emit('decline')">
						<XIcon />
						{{ formatMessage(messages.decline) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button type="button" @click="emit('continue')">
						<CheckIcon />
						{{ formatMessage(messages.continue) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>
