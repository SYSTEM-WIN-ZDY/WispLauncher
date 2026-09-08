<script setup lang="ts">
import { CompassIcon, RefreshCwIcon } from '@modrinth/assets'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import EmptyState from '#ui/components/base/EmptyState.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages, formatContentTypeSentence } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	noContentInstalled: {
		id: 'content.page-layout.empty.no-content-installed',
		defaultMessage: 'No content installed',
	},
	emptyHint: {
		id: 'content.page-layout.empty.hint',
		defaultMessage: 'Browse or upload {contentType} to get started',
	},
	browseContent: {
		id: 'content.page-layout.browse-content',
		defaultMessage: 'Browse content',
	},
})

const props = withDefaults(
	defineProps<{
		contentTypeLabel: string
		busy?: boolean
		busyTooltip?: string | null
		refreshing?: boolean
		disableAddContent?: boolean
		disableAddContentTooltip?: string
	}>(),
	{
		busy: false,
		busyTooltip: null,
		refreshing: false,
		disableAddContent: false,
		disableAddContentTooltip: undefined,
	},
)

const emit = defineEmits<{
	browse: []
	refresh: []
}>()
</script>

<template>
	<EmptyState type="empty-inbox">
		<template #heading>
			{{ formatMessage(messages.noContentInstalled) }}
		</template>
		<template #description>
			{{
				formatMessage(messages.emptyHint, {
					contentType: formatContentTypeSentence(
						formatMessage,
						props.contentTypeLabel,
						2,
						'content',
					),
				})
			}}
		</template>
		<template #actions>
			<ButtonStyled type="outlined">
				<button
					v-tooltip="props.busyTooltip"
					:disabled="props.refreshing"
					class="!h-10"
					@click="emit('refresh')"
				>
					<RefreshCwIcon :class="['size-5', { 'animate-spin': props.refreshing }]" />
					{{ formatMessage(commonMessages.refreshButton) }}
				</button>
			</ButtonStyled>
			<ButtonStyled color="brand">
				<button
					v-tooltip="
						props.busyTooltip ??
						(props.disableAddContent ? props.disableAddContentTooltip : undefined)
					"
					:disabled="props.busy || props.disableAddContent"
					class="!h-10 flex items-center gap-2"
					@click="emit('browse')"
				>
					<CompassIcon class="size-5" />
					<span>{{ formatMessage(messages.browseContent) }}</span>
				</button>
			</ButtonStyled>
		</template>
	</EmptyState>
</template>
