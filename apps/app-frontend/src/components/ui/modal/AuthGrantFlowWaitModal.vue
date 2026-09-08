<script setup lang="ts">
import { LogInIcon, SpinnerIcon } from '@modrinth/assets'
import { commonMessages, defineMessages, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

defineProps({
	onFlowCancel: {
		type: Function,
		default() {
			return async () => {}
		},
	},
})

const modal = ref()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	signInBrowser: {
		id: 'app.auth.sign-in-browser',
		defaultMessage: 'Please sign in in the browser window that just opened to continue.',
	},
})

function show() {
	modal.value.show()
}

function hide() {
	modal.value.hide()
}

defineExpose({ show, hide })
</script>
<template>
	<ModalWrapper ref="modal" @hide="onFlowCancel">
		<template #title>
			<span class="items-center gap-2 text-lg font-extrabold text-contrast">
				<LogInIcon /> {{ formatMessage(commonMessages.signInButton) }}
			</span>
		</template>

		<div class="flex justify-center gap-2">
			<SpinnerIcon class="w-12 h-12 animate-spin" />
		</div>
		<p class="text-sm text-secondary">
			{{ formatMessage(messages.signInBrowser) }}
		</p>
	</ModalWrapper>
</template>
