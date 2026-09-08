<script setup lang="ts">
import { ArrowDown, ArrowUp, ChevronRight, FileText } from 'lucide-vue-next'

import Badge from '~/components/ui/Badge.vue'
import type { ErrorRowDto, ErrorSort, SortDirection } from '~/shared/types/telemetry'
import { formatNumber, formatUtcDay } from '~/utils/format'

defineProps<{
	items: ErrorRowDto[]
	sort: ErrorSort
	direction: SortDirection
}>()

const emit = defineEmits<{
	select: [fingerprint: string]
	sort: [sort: ErrorSort]
}>()

const columns: Array<{ key: ErrorSort; label: string }> = [
	{ key: 'firstSeen', label: '首次出现' },
	{ key: 'lastSeen', label: '最近出现' },
	{ key: 'occurrences', label: '发生次数' },
	{ key: 'installations', label: '影响安装' },
]
</script>

<template>
	<div class="overflow-x-auto">
		<table class="w-full min-w-[1040px] table-fixed text-left text-sm">
			<thead class="border-b border-surface-4 bg-surface-3 text-xs text-muted-foreground">
				<tr>
					<th class="w-[25%] px-3 py-2.5 font-medium">错误指纹</th>
					<th class="w-[15%] px-3 py-2.5 font-medium">错误类型</th>
					<th class="w-[12%] px-3 py-2.5 font-medium">版本</th>
					<th v-for="column in columns" :key="column.key" class="w-[11%] px-3 py-2.5 font-medium">
						<button
							class="inline-flex cursor-pointer items-center gap-1 hover:text-foreground"
							@click="emit('sort', column.key)"
						>
							{{ column.label }}
							<component
								:is="direction === 'asc' ? ArrowUp : ArrowDown"
								v-if="sort === column.key"
								class="size-3"
							/>
						</button>
					</th>
					<th class="w-[10%] px-3 py-2.5 text-center font-medium">样本</th>
					<th class="w-10 px-2 py-2.5"><span class="sr-only">打开详情</span></th>
				</tr>
			</thead>
			<tbody>
				<tr
					v-for="item in items"
					:key="item.fingerprint"
					class="cursor-pointer border-b border-surface-4 transition-colors last:border-0 hover:bg-surface-3/70 active:bg-surface-4/60"
					tabindex="0"
					@click="emit('select', item.fingerprint)"
					@keydown.enter="emit('select', item.fingerprint)"
				>
					<td class="px-3 py-3 align-top">
						<p class="truncate font-mono text-xs font-medium" :title="item.fingerprint">
							{{ item.fingerprint }}
						</p>
						<p class="mt-1 truncate text-xs text-muted-foreground" :title="item.latestMessage">
							{{ item.latestMessage }}
						</p>
					</td>
					<td class="truncate px-3 py-3 align-top text-xs" :title="item.errorType">
						{{ item.errorType }}
					</td>
					<td class="truncate px-3 py-3 align-top font-mono text-xs">{{ item.appVersion }}</td>
					<td class="px-3 py-3 align-top text-xs tabular-nums">
						{{ formatUtcDay(item.firstSeen) }}
					</td>
					<td class="px-3 py-3 align-top text-xs tabular-nums">
						{{ formatUtcDay(item.lastSeen) }}
					</td>
					<td class="px-3 py-3 align-top text-xs tabular-nums">
						{{ formatNumber(item.occurrenceCount) }}
					</td>
					<td class="px-3 py-3 align-top text-xs tabular-nums">
						{{ formatNumber(item.affectedInstallations) }}
					</td>
					<td class="px-3 py-3 text-center align-top">
						<Badge :variant="item.hasSample ? 'success' : 'secondary'">
							<FileText class="mr-1 size-3" />{{ item.hasSample ? '可读取' : '无' }}
						</Badge>
					</td>
					<td class="px-2 py-3 align-top"><ChevronRight class="size-4 text-muted-foreground" /></td>
				</tr>
			</tbody>
		</table>
	</div>
</template>
