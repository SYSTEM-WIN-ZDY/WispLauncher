<script setup lang="ts">
import { Check, Copy } from 'lucide-vue-next'

import Alert from '~/components/ui/Alert.vue'
import Badge from '~/components/ui/Badge.vue'
import Button from '~/components/ui/Button.vue'
import Sheet from '~/components/ui/Sheet.vue'
import Skeleton from '~/components/ui/Skeleton.vue'
import Tooltip from '~/components/ui/Tooltip.vue'
import type { ErrorDetailDto, ErrorSampleDto } from '~/shared/types/telemetry'
import { formatNumber, formatUtcDay, formatUtcTimestamp } from '~/utils/format'

const props = defineProps<{
	open: boolean
	detail: ErrorDetailDto | null
	sample: ErrorSampleDto | null
	pending: boolean
	error: boolean
}>()
const emit = defineEmits<{ 'update:open': [value: boolean]; retry: [] }>()
const copied = ref(false)

async function copyFingerprint(): Promise<void> {
	if (!props.detail) return
	await navigator.clipboard.writeText(props.detail.fingerprint)
	copied.value = true
	setTimeout(() => (copied.value = false), 1_500)
}
</script>

<template>
	<Sheet
		:open="open"
		title="错误组详情"
		description="查看一个错误指纹及其最新登记样本。"
		@update:open="emit('update:open', $event)"
	>
		<div v-if="pending" class="space-y-4" data-state="loading">
			<Skeleton class="h-20 w-full" /><Skeleton class="h-40 w-full" /><Skeleton
				class="h-64 w-full"
			/>
		</div>
		<Alert v-else-if="error" variant="destructive" title="无法加载该错误组">
			<Button class="mt-3" size="sm" variant="outline" @click="emit('retry')">重新加载</Button>
		</Alert>
		<div v-else-if="detail" class="space-y-6">
			<section>
				<div class="flex items-start gap-2">
					<p class="min-w-0 flex-1 break-all font-mono text-sm font-medium">
						{{ detail.fingerprint }}
					</p>
					<Tooltip :text="copied ? '已复制' : '复制错误指纹'">
						<Button
							variant="outline"
							size="icon"
							aria-label="复制错误指纹"
							@click="copyFingerprint"
						>
							<Check v-if="copied" class="size-4 text-emerald-600" /><Copy v-else class="size-4" />
						</Button>
					</Tooltip>
				</div>
				<div class="mt-3 flex flex-wrap gap-2">
					<Badge>{{ detail.errorType }}</Badge>
					<Badge variant="secondary">{{ detail.appVersion }}</Badge>
					<Badge :variant="detail.hasSample ? 'success' : 'secondary'">
						{{ detail.hasSample ? '有可读样本' : '无样本' }}
					</Badge>
				</div>
			</section>

			<dl class="grid grid-cols-2 gap-x-5 gap-y-4 border-y py-4 text-sm">
				<div>
					<dt class="text-xs text-muted-foreground">首次出现</dt>
					<dd class="mt-1 tabular-nums">{{ formatUtcDay(detail.firstSeen) }}（UTC）</dd>
				</div>
				<div>
					<dt class="text-xs text-muted-foreground">最近出现</dt>
					<dd class="mt-1 tabular-nums">{{ formatUtcDay(detail.lastSeen) }}（UTC）</dd>
				</div>
				<div>
					<dt class="text-xs text-muted-foreground">发生次数</dt>
					<dd class="mt-1 tabular-nums">{{ formatNumber(detail.occurrenceCount) }}</dd>
				</div>
				<div>
					<dt class="text-xs text-muted-foreground">影响安装数</dt>
					<dd class="mt-1 tabular-nums">{{ formatNumber(detail.affectedInstallations) }}</dd>
				</div>
			</dl>

			<section>
				<h3 class="text-xs font-semibold uppercase text-muted-foreground">最新脱敏消息</h3>
				<p class="mt-2 whitespace-pre-wrap break-words text-sm">{{ detail.latestMessage }}</p>
			</section>

			<Alert v-if="!sample" title="没有登记样本"> 该错误指纹没有可读取的 R2 上下文样本。 </Alert>
			<template v-else>
				<section>
					<h3 class="text-xs font-semibold uppercase text-muted-foreground">已登记的 R2 样本</h3>
					<p class="mt-1 text-xs text-muted-foreground">
						{{ formatUtcTimestamp(sample.occurredAt) }}
					</p>
					<dl class="mt-3 grid grid-cols-1 gap-3 text-sm sm:grid-cols-2">
						<div>
							<dt class="text-xs text-muted-foreground">路由</dt>
							<dd class="mt-1 break-all">{{ sample.route || '暂无' }}</dd>
						</div>
						<div>
							<dt class="text-xs text-muted-foreground">命令</dt>
							<dd class="mt-1 break-all">{{ sample.command || '暂无' }}</dd>
						</div>
						<div>
							<dt class="text-xs text-muted-foreground">操作系统</dt>
							<dd class="mt-1">{{ sample.platform }}</dd>
						</div>
						<div>
							<dt class="text-xs text-muted-foreground">CPU 架构</dt>
							<dd class="mt-1">{{ sample.architecture }}</dd>
						</div>
					</dl>
				</section>
				<section v-if="sample.stack">
					<h3 class="text-xs font-semibold text-muted-foreground">脱敏调用栈</h3>
					<pre
						class="mt-2 max-h-80 overflow-auto whitespace-pre-wrap break-words rounded-lg border bg-muted/45 p-3 font-mono text-xs"
						>{{ sample.stack }}</pre
					>
				</section>
				<section v-if="sample.context">
					<h3 class="text-xs font-semibold text-muted-foreground">脱敏上下文</h3>
					<pre
						class="mt-2 max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-lg border bg-muted/45 p-3 font-mono text-xs"
						>{{ sample.context }}</pre
					>
				</section>
			</template>
		</div>
	</Sheet>
</template>
