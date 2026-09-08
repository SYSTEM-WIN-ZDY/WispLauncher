<template>
	<div v-if="allTags.length > 0" class="flex flex-col gap-3">
		<h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
		<div class="flex flex-wrap gap-1">
			<TagItem
				v-for="tag in allTags"
				:key="tag"
				:action="props.tagAction ? () => props.tagAction?.(tag) : undefined"
			>
				<FormattedTag :tag="tag" />
			</TagItem>
		</div>
	</div>
</template>
<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../composables'
import FormattedTag from '../base/FormattedTag.vue'
import TagItem from '../base/TagItem.vue'

const props = defineProps<{
	project: {
		categories: string[]
		additional_categories: string[]
	}
	tagAction?: (tag: string) => void
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.about.tags.title',
		defaultMessage: 'Tags',
	},
})

const allTags = computed(() => [
	...props.project.categories,
	...props.project.additional_categories,
])
</script>
