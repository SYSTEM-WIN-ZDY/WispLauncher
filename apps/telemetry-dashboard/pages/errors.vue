<script setup lang="ts">
import { ArrowDownUp, ChevronLeft, ChevronRight, RotateCcw, Search } from 'lucide-vue-next'

import AppState from '~/components/AppState.vue'
import ErrorDetailSheet from '~/components/errors/ErrorDetailSheet.vue'
import ErrorsTable from '~/components/errors/ErrorsTable.vue'
import PageHeader from '~/components/PageHeader.vue'
import Alert from '~/components/ui/Alert.vue'
import Button from '~/components/ui/Button.vue'
import Card from '~/components/ui/Card.vue'
import Select from '~/components/ui/Select.vue'
import Skeleton from '~/components/ui/Skeleton.vue'
import Tooltip from '~/components/ui/Tooltip.vue'
import type {
	ErrorDetailDto,
	ErrorSampleDto,
	ErrorSort,
	ErrorsPageDto,
	SortDirection,
} from '~/shared/types/telemetry'
import { formatNumber } from '~/utils/format'

const range = useDashboardRange()
const refreshEpoch = useDashboardRefresh()
const requestFetch = useRequestFetch()

const searchInput = ref('')
const search = ref('')
const version = ref('')
const platform = ref('')
const errorType = ref('')
const hasSample = ref('')
const page = ref(1)
const pageSize = ref('25')
const sort = ref<ErrorSort>('lastSeen')
const direction = ref<SortDirection>('desc')
let searchTimer: ReturnType<typeof setTimeout> | null = null

watch(searchInput, (value) => {
	if (searchTimer) clearTimeout(searchTimer)
	searchTimer = setTimeout(() => {
		search.value = value.trim()
		page.value = 1
	}, 250)
})
watch([range, version, platform, errorType, hasSample, pageSize], () => (page.value = 1))

const query = computed(() => ({
	range: range.value,
	page: page.value,
	pageSize: Number(pageSize.value),
	search: search.value || undefined,
	version: version.value || undefined,
	platform: platform.value || undefined,
	errorType: errorType.value || undefined,
	hasSample: hasSample.value || undefined,
	sort: sort.value,
	direction: direction.value,
}))

const { data, status, error, refresh } = await useAsyncData(
	'errors-data',
	() => requestFetch<ErrorsPageDto>('/api/admin/errors', { query: query.value }),
	{ watch: [query, refreshEpoch] },
)

const selected = ref<string | null>(null)
const detail = ref<ErrorDetailDto | null>(null)
const sample = ref<ErrorSampleDto | null>(null)
const detailPending = ref(false)
const detailError = ref(false)

async function openDetail(fingerprint: string): Promise<void> {
	selected.value = fingerprint
	detail.value = null
	sample.value = null
	detailPending.value = true
	detailError.value = false
	const encoded = encodeURIComponent(fingerprint)
	const [detailResult, sampleResult] = await Promise.allSettled([
		requestFetch<ErrorDetailDto>(`/api/admin/errors/${encoded}`),
		requestFetch<ErrorSampleDto>(`/api/admin/errors/${encoded}/sample`),
	])
	if (selected.value !== fingerprint) return
	if (detailResult.status === 'fulfilled') detail.value = detailResult.value
	else detailError.value = true
	if (sampleResult.status === 'fulfilled') sample.value = sampleResult.value
	detailPending.value = false
}

function changeSort(next: ErrorSort): void {
	if (sort.value === next) direction.value = direction.value === 'asc' ? 'desc' : 'asc'
	else {
		sort.value = next
		direction.value = 'desc'
	}
	page.value = 1
}

function resetFilters(): void {
	searchInput.value = ''
	search.value = ''
	version.value = ''
	platform.value = ''
	errorType.value = ''
	hasSample.value = ''
	page.value = 1
}

const activeFilters = computed(() =>
	Boolean(search.value || version.value || platform.value || errorType.value || hasSample.value),
)
const withAll = (items: string[], label: string) => [
	{ value: '', label },
	...items.map((item) => ({ value: item, label: item })),
]
</script>

<template>
	<div>
		<PageHeader
			title="错误分析"
			description="搜索、筛选并查看经过脱敏处理的聚合错误组和单个上下文样本。"
		/>

		<Card class="overflow-hidden">
			<div class="flex flex-wrap items-center gap-2 border-b border-surface-4 bg-surface-3/35 p-3">
				<label class="relative min-w-56 flex-1 lg:max-w-sm">
					<span class="sr-only">搜索错误</span>
					<Search
						class="pointer-events-none absolute left-3 top-2.5 size-4 text-muted-foreground"
					/>
					<input
						v-model="searchInput"
						class="h-9 w-full rounded-md border border-surface-4 bg-surface-2 pl-9 pr-3 text-sm outline-none transition-colors placeholder:text-muted-foreground hover:border-surface-5 focus:ring-2 focus:ring-ring"
						placeholder="搜索指纹、类型或消息"
						maxlength="120"
					/>
				</label>
				<Select
					v-model="version"
					label="启动器版本"
					:options="withAll(data?.filters.versions || [], '全部版本')"
				/>
				<Select
					v-model="platform"
					label="操作系统"
					:options="withAll(data?.filters.platforms || [], '全部系统')"
				/>
				<Select
					v-model="errorType"
					label="错误类型"
					:options="withAll(data?.filters.errorTypes || [], '全部类型')"
				/>
				<Select
					v-model="hasSample"
					label="样本状态"
					:options="[
						{ value: '', label: '全部样本状态' },
						{ value: 'true', label: '有样本' },
						{ value: 'false', label: '无样本' },
					]"
				/>
				<Tooltip text="清除筛选">
					<Button
						variant="ghost"
						size="icon"
						:disabled="!activeFilters"
						aria-label="清除筛选"
						@click="resetFilters"
					>
						<RotateCcw class="size-4" />
					</Button>
				</Tooltip>
			</div>

			<div v-if="status === 'pending' && !data" class="space-y-2 p-3" data-state="loading">
				<Skeleton class="h-10 w-full" />
				<Skeleton v-for="index in 8" :key="index" class="h-14 w-full" />
			</div>
			<AppState v-else-if="error && !data" compact kind="error" @retry="refresh" />
			<template v-else-if="data">
				<Alert v-if="error" variant="destructive" title="刷新失败" class="m-3">
					当前仍显示上一次成功获取的结果。
					<button class="cursor-pointer font-medium underline" @click="() => refresh()">
						重新加载
					</button>
				</Alert>
				<div
					v-if="!data.items.length"
					class="flex flex-col items-center justify-center px-4 py-16 text-center"
					:data-state="activeFilters ? 'filtered-empty' : 'empty'"
				>
					<div class="flex size-10 items-center justify-center rounded-lg bg-muted">
						<ArrowDownUp class="size-5 text-muted-foreground" />
					</div>
					<h2 class="mt-4 text-sm font-semibold">
						{{ activeFilters ? '没有符合条件的错误组' : '当前 UTC 时间范围内没有错误组' }}
					</h2>
					<p class="mt-1 text-sm text-muted-foreground">
						{{ activeFilters ? '请调整或清除当前筛选条件。' : '这段时间内没有聚合到错误。' }}
					</p>
					<Button
						v-if="activeFilters"
						class="mt-4"
						variant="outline"
						size="sm"
						@click="resetFilters"
						>清除筛选</Button
					>
				</div>
				<div v-else :class="status === 'pending' && 'opacity-65'" :aria-busy="status === 'pending'">
					<ErrorsTable
						:items="data.items"
						:sort="sort"
						:direction="direction"
						@select="openDetail"
						@sort="changeSort"
					/>
				</div>
				<footer
					class="flex flex-wrap items-center justify-between gap-3 border-t px-3 py-2.5 text-xs text-muted-foreground"
				>
					<p>
						<span class="font-medium tabular-nums text-foreground">{{
							formatNumber(data.total)
						}}</span>
						个错误组
					</p>
					<div class="flex items-center gap-2">
						<Select
							v-model="pageSize"
							label="每页行数"
							:options="[
								{ value: '25', label: '25 条/页' },
								{ value: '50', label: '50 条/页' },
								{ value: '100', label: '100 条/页' },
							]"
						/>
						<span class="w-24 text-center tabular-nums"
							>第 {{ data.page }} / {{ data.totalPages }} 页</span
						>
						<Button
							variant="outline"
							size="icon"
							:disabled="page <= 1 || status === 'pending'"
							aria-label="上一页"
							@click="page--"
						>
							<ChevronLeft class="size-4" />
						</Button>
						<Button
							variant="outline"
							size="icon"
							:disabled="page >= data.totalPages || status === 'pending'"
							aria-label="下一页"
							@click="page++"
						>
							<ChevronRight class="size-4" />
						</Button>
					</div>
				</footer>
			</template>
		</Card>

		<ErrorDetailSheet
			:open="Boolean(selected)"
			:detail="detail"
			:sample="sample"
			:pending="detailPending"
			:error="detailError"
			@retry="selected && openDetail(selected)"
			@update:open="!$event && (selected = null)"
		/>
	</div>
</template>
