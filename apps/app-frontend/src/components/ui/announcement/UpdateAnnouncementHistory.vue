<script setup lang="ts">
import { CalendarIcon, HistoryIcon, RefreshCwIcon } from '@modrinth/assets'
import { Accordion, NewButton as Button, defineMessages, TagItem, useVIntl } from '@modrinth/ui'
import { renderHighlightedString } from '@modrinth/utils'
import { computed, ref } from 'vue'

import {
	getAnnouncementByVersion,
	getAnnouncements,
	getLocalizedAnnouncementText,
	type LauncherAnnouncement,
} from '@/announcements/catalog'
import { WispBrandConfig } from '@/config'
import i18n from '@/i18n.config'
import { compareSemanticVersions, parseVersion } from '@/helpers/version-compatibility'

import UpdateAnnouncementContent from './UpdateAnnouncementContent.vue'

const props = defineProps<{
	currentVersion: string
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'app.settings.updates.announcements.title',
		defaultMessage: 'Update announcements',
	},
	description: {
		id: 'app.settings.updates.announcements.description',
		defaultMessage: 'See what changed in this version and browse previous releases.',
	},
	history: {
		id: 'app.settings.updates.announcements.history',
		defaultMessage: 'Version history',
	},
	empty: {
		id: 'app.settings.updates.announcements.empty',
		defaultMessage: 'No bundled update announcements are available.',
	},
	loading: {
		id: 'app.settings.updates.announcements.loading',
		defaultMessage: 'Loading release history…',
	},
	error: {
		id: 'app.settings.updates.announcements.error',
		defaultMessage: 'Could not fetch release history.',
	},
	retry: {
		id: 'app.settings.updates.announcements.retry',
		defaultMessage: 'Retry',
	},
	source: {
		id: 'app.settings.updates.announcements.source',
		defaultMessage: 'Fetched from {source}',
	},
	version: {
		id: 'app.update-announcement.version',
		defaultMessage: 'Version {version}',
	},
})

const locale = computed(() => i18n.global.locale.value)
const launcherAnnouncements = getAnnouncements()

// GitHub releases fetched at runtime. Each entry carries the raw markdown body
// plus the version parsed from its tag so history can be ordered numerically.
interface GithubReleaseEntry {
	id: string
	version: string
	publishedAt: string
	name: string
	body: string
	htmlUrl: string
}

const GITHUB_RELEASES_ENDPOINT =
	'https://api.github.com/repos/SYSTEM-WIN-ZDY/WispLauncher/releases?per_page=100'
const GITEE_RELEASES_ENDPOINT =
	'https://gitee.com/api/v5/repos/system-win-zdy/WispLauncher/releases?per_page=100'
const GITEE_RELEASE_URL_PREFIX = 'https://gitee.com/system-win-zdy/WispLauncher/releases/tag/'

const releases = ref<GithubReleaseEntry[]>([])
const loading = ref(true)
const error = ref(false)
const activeSource = ref<'GitHub' | 'Gitee' | null>(null)

function extractVersion(tagName: string, name: string): string {
	const candidates = [tagName, name]
	for (const candidate of candidates) {
		const match = candidate?.match(/(\d+\.\d+(?:\.\d+)?)/)
		if (match) return match[1]
	}
	return tagName.replace(/^v/i, '').trim()
}

function sortByVersionDescending(a: GithubReleaseEntry, b: GithubReleaseEntry): number {
	const comparison = compareSemanticVersions(a.version, b.version)
	if (comparison !== null && comparison !== 0) return -comparison
	return b.publishedAt.localeCompare(a.publishedAt)
}

async function fetchGithubReleases(): Promise<GithubReleaseEntry[]> {
	const response = await fetch(GITHUB_RELEASES_ENDPOINT, {
		headers: { Accept: 'application/vnd.github+json' },
	})
	if (!response.ok) throw new Error(`GitHub API returned ${response.status}`)

	const payload = (await response.json()) as Array<{
		id: number
		tag_name: string
		name: string
		published_at: string | null
		body: string | null
		html_url: string
	}>

	return payload
		.filter((release) => release && parseVersion(extractVersion(release.tag_name ?? '', release.name ?? '')))
		.map((release) => {
			const version = extractVersion(release.tag_name ?? '', release.name ?? '')
			const fallbackName =
				release.name?.trim() && release.name !== 'Release' ? release.name.trim() : `v${version}`
			return {
				id: String(release.id),
				version,
				publishedAt: release.published_at ?? '',
				name: fallbackName || `v${version}`,
				body: release.body ?? '',
				htmlUrl: release.html_url,
			}
		})
		.sort(sortByVersionDescending)
}

// Gitee release API responses differ from GitHub's: releases expose
// `created_at` instead of `published_at`, and there is no release-level
// `html_url`, so the web URL is derived from the tag name.
async function fetchGiteeReleases(): Promise<GithubReleaseEntry[]> {
	const response = await fetch(GITEE_RELEASES_ENDPOINT)
	if (!response.ok) throw new Error(`Gitee API returned ${response.status}`)

	const payload = (await response.json()) as Array<{
		id: number
		tag_name: string
		name: string
		created_at: string | null
		body: string | null
	}>

	return payload
		.filter((release) => release && parseVersion(extractVersion(release.tag_name ?? '', release.name ?? '')))
		.map((release) => {
			const version = extractVersion(release.tag_name ?? '', release.name ?? '')
			const fallbackName =
				release.name?.trim() && release.name !== 'Release' ? release.name.trim() : `v${version}`
			return {
				id: String(release.id),
				version,
				publishedAt: release.created_at ?? '',
				name: fallbackName || `v${version}`,
				body: release.body ?? '',
				htmlUrl: `${GITEE_RELEASE_URL_PREFIX}${encodeURIComponent(release.tag_name ?? '')}`,
			}
		})
		.sort(sortByVersionDescending)
}

// Try GitHub first, then fall back to Gitee. Whichever source answers first
// wins; only when both fail is the error state shown.
async function loadReleases() {
	loading.value = true
	error.value = false
	try {
		releases.value = await fetchGithubReleases().then((entries) => {
			activeSource.value = 'GitHub'
			return entries
		})
	} catch (githubError) {
		console.warn('GitHub release history fetch failed, trying Gitee:', githubError)
		try {
			releases.value = await fetchGiteeReleases().then((entries) => {
				activeSource.value = 'Gitee'
				return entries
			})
		} catch (giteeError) {
			console.warn('Gitee release history fetch failed too:', giteeError)
			error.value = true
		}
	} finally {
		loading.value = false
	}
}

loadReleases()

const currentAnnouncement = computed(() => getAnnouncementByVersion(props.currentVersion))
const currentRelease = computed(() =>
	releases.value.find((release) => release.version === props.currentVersion),
)

function releaseToAnnouncement(release: GithubReleaseEntry): LauncherAnnouncement {
	return {
		id: release.id,
		version: release.version,
		publishedAt: release.publishedAt,
		title: {
			'en-US': release.name,
			'zh-CN': release.name,
		},
		changes: {},
		notes: {
			'en-US': release.body,
			'zh-CN': release.body,
		},
		externalUrl: release.htmlUrl,
	}
}

// History = every fetched GitHub release except the current version. Releases
// without a bundled announcement fall back to the raw GitHub changelog body.
const historyAnnouncements = computed<LauncherAnnouncement[]>(() => {
	const notCurrent = releases.value.filter(
		(release) => release.version !== props.currentVersion,
	)
	return notCurrent.map(releaseToAnnouncement)
})

function renderReleaseBody(announcement: LauncherAnnouncement): string {
	const text = getLocalizedAnnouncementText(announcement.notes ?? {}, locale.value)
	if (!text) return ''
	// GitHub release bodies can be raw HTML fragments (e.g. copied from a web
	// page); render those as-is and only highlight markdown for plain text.
	if (/^\s*</.test(text)) return text
	return renderHighlightedString(text)
}
</script>

<template>
	<section class="update-announcement-history">
		<div class="flex min-w-0 flex-col gap-1">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.title) }}
			</h2>
			<p class="m-0 leading-relaxed text-secondary">
				{{ formatMessage(messages.description) }}
			</p>
		</div>

		<div class="min-w-0">
			<template v-if="currentAnnouncement">
				<UpdateAnnouncementContent
					:announcement="currentAnnouncement"
					:version="currentVersion"
					:external-url="currentAnnouncement?.externalUrl ?? WispBrandConfig.website"
				/>
			</template>
			<template v-else-if="currentRelease">
				<div class="flex flex-col gap-2">
					<header class="flex min-w-0 flex-col gap-2">
						<h2 class="m-0 break-words text-xl font-semibold text-contrast">
							{{ currentRelease.name }}
						</h2>
						<div class="flex flex-wrap items-center gap-2 text-sm text-secondary">
							<span>{{ formatMessage(messages.version, { version: currentRelease.version }) }}</span>
							<span class="flex items-center gap-1">
								<CalendarIcon aria-hidden="true" class="size-3.5" />
								<time :datetime="currentRelease.publishedAt">{{ currentRelease.publishedAt }}</time>
							</span>
						</div>
					</header>
					<!-- eslint-disable-next-line vue/no-v-html -->
					<div class="markdown-body" v-html="renderReleaseBody(releaseToAnnouncement(currentRelease))" />
				</div>
			</template>
			<UpdateAnnouncementContent
				v-else
				:version="currentVersion"
				:external-url="WispBrandConfig.website"
			/>
		</div>

		<div class="flex min-w-0 flex-col gap-3">
			<h3 class="m-0 flex items-center gap-2 text-base font-semibold text-contrast">
				<HistoryIcon aria-hidden="true" class="size-4 text-secondary" />
				{{ formatMessage(messages.history) }}
				<span
					v-if="!loading && !error && activeSource"
					class="ml-auto flex-shrink-0 text-xs font-normal text-secondary"
				>
					{{ formatMessage(messages.source, { source: activeSource }) }}
				</span>
			</h3>

			<div v-if="loading" class="flex items-center gap-2 text-sm text-secondary">
				<RefreshCwIcon class="size-4 animate-spin" />
				{{ formatMessage(messages.loading) }}
			</div>

			<div v-else-if="error" class="flex flex-col items-start gap-3 text-sm text-secondary">
				<p class="m-0">{{ formatMessage(messages.error) }}</p>
				<Button type="outlined" @click="loadReleases">
					{{ formatMessage(messages.retry) }}
				</Button>
			</div>

			<p v-else-if="historyAnnouncements.length === 0" class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.empty) }}
			</p>
			<div v-else class="flex min-w-0 flex-col gap-2">
				<Accordion
					v-for="announcement in historyAnnouncements"
					:key="announcement.id"
					class="update-announcement-history-item hover:border-surface-4 focus-within:border-surface-4"
					button-class="group flex w-full cursor-pointer items-center gap-3 border-0 bg-transparent px-4 py-3 text-left"
				>
					<template #title>
						<div class="flex min-w-0 flex-1 items-center gap-3">
							<div class="flex min-w-0 flex-1 flex-col gap-1">
								<span
									class="truncate font-semibold text-primary transition-colors group-hover:text-contrast"
								>
									{{ getLocalizedAnnouncementText(announcement.title, locale) }}
								</span>
								<div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-secondary">
									<TagItem class="px-1.5 py-0.5 text-xs">v{{ announcement.version }}</TagItem>
									<span class="flex items-center gap-1">
										<CalendarIcon aria-hidden="true" class="size-3.5" />
										<time :datetime="announcement.publishedAt">{{ announcement.publishedAt }}</time>
									</span>
								</div>
							</div>
						</div>
					</template>
					<div class="update-announcement-history-item-content">
						<div v-if="getAnnouncementByVersion(announcement.version)">
							<UpdateAnnouncementContent
								:announcement="getAnnouncementByVersion(announcement.version)"
								:show-header="false"
								:external-url="announcement.externalUrl"
							/>
						</div>
						<!-- eslint-disable-next-line vue/no-v-html -->
						<div v-else class="markdown-body" v-html="renderReleaseBody(announcement)" />
					</div>
				</Accordion>
			</div>
		</div>
	</section>
</template>

<style scoped>
.update-announcement-history {
	display: flex;
	min-width: 0;
	flex-direction: column;
	gap: var(--gap-xl);
	padding: var(--gap-xl);
	border: 1px solid
		var(--settings-card-border, color-mix(in srgb, var(--surface-4) 72%, transparent));
	border-radius: var(--radius-md);
	background: var(--surface-2);
}

.update-announcement-history-item {
	min-width: 0;
	overflow: hidden;
	border: 1px solid
		var(--settings-card-border, color-mix(in srgb, var(--surface-4) 72%, transparent));
	border-radius: var(--radius-sm);
	background: var(--surface-3);
	transition: border-color 120ms ease;
}

.update-announcement-history-item-content {
	padding: var(--gap-lg);
	border-top: 1px solid
		var(--settings-divider, color-mix(in srgb, var(--surface-4) 55%, transparent));
}
</style>
