<script setup lang="ts">
import { Archive, Database, FolderArchive, Gauge, ShieldCheck, TimerReset } from 'lucide-vue-next'

import AppState from '~/components/AppState.vue'
import PageHeader from '~/components/PageHeader.vue'
import StatusRow from '~/components/system/StatusRow.vue'
import Alert from '~/components/ui/Alert.vue'
import Badge from '~/components/ui/Badge.vue'
import Card from '~/components/ui/Card.vue'
import Skeleton from '~/components/ui/Skeleton.vue'
import type { SystemDto } from '~/shared/types/telemetry'
import { formatNumber, formatUtcTimestamp } from '~/utils/format'

const refreshEpoch = useDashboardRefresh()
const requestFetch = useRequestFetch()
const { data, status, error, refresh } = await useAsyncData(
	'system-data',
	() => requestFetch<SystemDto>('/api/admin/system'),
	{ watch: [refreshEpoch] },
)

const retentionRows = computed(() =>
	data.value
		? [
				{
					label: 'daily_active',
					value: `${data.value.limits.dailyActiveRetentionDays} 天`,
					icon: Database,
				},
				{
					label: 'error_reports',
					value: `${data.value.limits.errorReportsRetentionDays} 天`,
					icon: Archive,
				},
				{
					label: 'R2 错误上下文',
					value: `${data.value.limits.r2RetentionDays} 天`,
					icon: FolderArchive,
				},
				{
					label: 'error_groups / error_daily',
					value: `${data.value.limits.errorAggregatesRetentionDays} 天`,
					icon: TimerReset,
				},
			]
		: [],
)
</script>

<template>
	<div>
		<PageHeader
			title="系统状态"
			description="检查采集服务、存储绑定、聚合任务、配额和数据保留策略。"
		/>

		<div
			v-if="status === 'pending' && !data"
			class="grid gap-4 lg:grid-cols-2"
			data-state="loading"
		>
			<Skeleton v-for="index in 4" :key="index" class="h-64" />
		</div>
		<AppState v-else-if="error || !data" compact kind="error" @retry="refresh" />
		<template v-else>
			<Alert
				v-if="data.r2Budget.used >= data.r2Budget.limit"
				variant="destructive"
				title="R2 对象预算已用尽"
				class="mb-5"
			>
				今日 UTC 配额已经耗尽。错误聚合仍会继续，但今天不再写入新的上下文样本。
			</Alert>
			<div
				class="grid gap-4 lg:grid-cols-2"
				:class="status === 'pending' && 'opacity-65'"
				:aria-busy="status === 'pending'"
			>
				<Card class="overflow-hidden">
					<div class="border-b px-4 py-3">
						<h2 class="text-sm font-semibold">服务检查</h2>
						<p class="mt-0.5 text-xs text-muted-foreground">
							检查时间 {{ formatUtcTimestamp(data.generatedAt) }}
						</p>
					</div>
					<StatusRow name="公开遥测 Worker" :check="data.publicWorker" />
					<StatusRow name="D1 只读查询" :check="data.d1" />
					<StatusRow name="R2 样本读取" :check="data.r2" />
					<StatusRow name="Cron 聚合任务" :check="data.cron" />
					<StatusRow name="Cloudflare 账户用量" :check="data.accountUsage" />
				</Card>

				<Card class="p-4">
					<div class="flex items-start justify-between gap-3">
						<div>
							<h2 class="text-sm font-semibold">错误上下文预算</h2>
							<p class="mt-1 text-xs text-muted-foreground">今日 UTC 对象预留使用情况。</p>
						</div>
						<Gauge class="size-5 text-muted-foreground" />
					</div>
					<div class="mt-7 flex items-end justify-between gap-4">
						<p class="text-3xl font-semibold tabular-nums">
							{{ formatNumber(data.r2Budget.used) }}
							<span class="text-base font-normal text-muted-foreground"
								>/ {{ formatNumber(data.r2Budget.limit) }}</span
							>
						</p>
						<Badge :variant="data.r2Budget.used >= data.r2Budget.limit ? 'destructive' : 'success'">
							{{ data.r2Budget.used >= data.r2Budget.limit ? '已达上限' : '配额正常' }}
						</Badge>
					</div>
					<div class="mt-4 h-2 overflow-hidden rounded-sm bg-muted">
						<div
							class="h-full rounded-sm"
							:class="data.r2Budget.used >= data.r2Budget.limit ? 'bg-destructive' : 'bg-primary'"
							:style="{
								width: `${Math.min(100, (data.r2Budget.used / data.r2Budget.limit) * 100)}%`,
							}"
						></div>
					</div>
					<dl class="mt-7 grid grid-cols-2 gap-4 border-t pt-4 text-sm">
						<div>
							<dt class="text-xs text-muted-foreground">STORE_ERROR_CONTEXT</dt>
							<dd class="mt-1 font-medium">
								{{ data.storeErrorContext ? 'Enabled' : 'Disabled' }}
							</dd>
						</div>
						<div>
							<dt class="text-xs text-muted-foreground">每组样本上限</dt>
							<dd class="mt-1 font-medium tabular-nums">{{ data.limits.samplesPerGroup }}</dd>
						</div>
						<div>
							<dt class="text-xs text-muted-foreground">最近数据日期</dt>
							<dd class="mt-1 font-medium tabular-nums">
								{{ data.latestDataDay ? `${data.latestDataDay}（UTC）` : '暂无' }}
							</dd>
						</div>
						<div>
							<dt class="text-xs text-muted-foreground">管理 API 模式</dt>
							<dd class="mt-1 inline-flex items-center gap-1.5 font-medium">
								<ShieldCheck class="size-4 text-emerald-600" />只读
							</dd>
						</div>
					</dl>
				</Card>

				<Card class="overflow-hidden lg:col-span-2">
					<div class="border-b px-4 py-3">
						<h2 class="text-sm font-semibold">数据保留策略</h2>
						<p class="mt-0.5 text-xs text-muted-foreground">生产环境当前配置的保留时间窗口。</p>
					</div>
					<div class="grid sm:grid-cols-2 xl:grid-cols-4">
						<div
							v-for="row in retentionRows"
							:key="row.label"
							class="flex items-center gap-3 border-b p-4 last:border-0 sm:border-r xl:border-b-0"
						>
							<div class="flex size-8 items-center justify-center rounded-md bg-muted">
								<component :is="row.icon" class="size-4 text-muted-foreground" />
							</div>
							<div class="min-w-0">
								<p class="truncate font-mono text-xs">{{ row.label }}</p>
								<p class="mt-1 text-sm font-medium">{{ row.value }}</p>
							</div>
						</div>
					</div>
				</Card>
			</div>
		</template>
	</div>
</template>
