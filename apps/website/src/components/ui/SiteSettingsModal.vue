<script setup lang="ts">
import { LanguagesIcon, PaintbrushIcon } from '@modrinth/assets'
import SettingsIcon from '@modrinth/assets/icons/settings.svg?component'
import XIcon from '@modrinth/assets/icons/x.svg?component'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import Toggle from '@modrinth/ui/src/components/base/Toggle.vue'
import LanguageSelector from '@modrinth/ui/src/components/settings/LanguageSelector.vue'
import ThemeSelector from '@modrinth/ui/src/components/settings/ThemeSelector.vue'
import { defineMessages, LOCALES, useVIntl } from '@modrinth/ui/src/composables/i18n.ts'
import { injectI18n } from '@modrinth/ui/src/providers/i18n.ts'

type Theme = 'system' | 'light' | 'dark' | 'oled'
type SettingsTab = 'appearance' | 'language'

const open = defineModel<boolean>({ required: true })
const { formatMessage } = useVIntl()
const { locale, setLocale } = injectI18n()

const preferredTheme = ref<Theme>('system')
const systemTheme = ref<'light' | 'dark'>('dark')
const advancedRendering = ref(true)
const reduceMotion = ref(false)
const externalLinksNewTab = ref(true)
const themeOptions = ['dark', 'light', 'oled', 'system'] as const
const supportedLocales = LOCALES.filter((item) => ['en-US', 'zh-CN'].includes(item.code))
const changingLocale = ref(false)
const activeTab = ref<SettingsTab>('appearance')

const messages = defineMessages({
	title: { id: 'wisp-settings.title', defaultMessage: 'Display settings' },
	description: {
		id: 'wisp-settings.description',
		defaultMessage: 'Customize how Wisp looks and reads on this device.',
	},
	close: { id: 'wisp-settings.close', defaultMessage: 'Close settings' },
	appearanceTitle: { id: 'wisp-settings.appearance.title', defaultMessage: 'Appearance' },
	languageTitle: { id: 'wisp-settings.language.title', defaultMessage: 'Language' },
	languageDescription: {
		id: 'wisp-settings.language.description',
		defaultMessage: 'Choose the language used by this website.',
	},
	themeTitle: { id: 'wisp-settings.theme.title', defaultMessage: 'Color theme' },
	themeDescription: {
		id: 'wisp-settings.theme.description',
		defaultMessage: 'Select your preferred color theme for Wisp on this device.',
	},
	interfaceTitle: { id: 'wisp-settings.interface.title', defaultMessage: 'Interface' },
	interfaceDescription: {
		id: 'wisp-settings.interface.description',
		defaultMessage: 'Enable or disable visual behavior on this device.',
	},
	advancedRenderingTitle: {
		id: 'wisp-settings.advanced-rendering.title',
		defaultMessage: 'Advanced rendering',
	},
	advancedRenderingDescription: {
		id: 'wisp-settings.advanced-rendering.description',
		defaultMessage: 'Use blur, gradients, and enhanced background effects.',
	},
	reduceMotionTitle: {
		id: 'wisp-settings.reduce-motion.title',
		defaultMessage: 'Reduce motion',
	},
	reduceMotionDescription: {
		id: 'wisp-settings.reduce-motion.description',
		defaultMessage: 'Disable decorative movement and transition effects.',
	},
	externalLinksTitle: {
		id: 'wisp-settings.external-links.title',
		defaultMessage: 'Open external links in new tab',
	},
	externalLinksDescription: {
		id: 'wisp-settings.external-links.description',
		defaultMessage: 'Keep the download page open when visiting another website.',
	},
	done: { id: 'wisp-settings.done', defaultMessage: 'Done' },
})

let systemThemeQuery: MediaQueryList | undefined

function applyTheme() {
	if (!import.meta.client) return

	const resolvedTheme = preferredTheme.value === 'system' ? systemTheme.value : preferredTheme.value
	document.documentElement.classList.remove('light-mode', 'dark-mode', 'oled-mode')
	document.documentElement.classList.add(`${resolvedTheme}-mode`, 'accent-pink')
	document.documentElement.style.colorScheme = resolvedTheme === 'light' ? 'light' : 'dark'
	localStorage.setItem('wisp-theme', preferredTheme.value)
}

function updateColorTheme(theme: Theme) {
	preferredTheme.value = theme
	applyTheme()
}

async function updateLocale(newLocale: string) {
	if (newLocale === locale.value) return
	changingLocale.value = true
	try {
		await setLocale(newLocale)
	} finally {
		changingLocale.value = false
	}
}

function applyRenderingPreferences() {
	if (!import.meta.client) return
	document.documentElement.classList.toggle('reduced-effects', !advancedRendering.value)
	document.documentElement.classList.toggle('reduced-motion', reduceMotion.value)
	localStorage.setItem('wisp-advanced-rendering', String(advancedRendering.value))
	localStorage.setItem('wisp-reduce-motion', String(reduceMotion.value))
	localStorage.setItem('wisp-external-links-new-tab', String(externalLinksNewTab.value))
}

function handleSystemTheme(event: MediaQueryListEvent) {
	systemTheme.value = event.matches ? 'dark' : 'light'
	if (preferredTheme.value === 'system') applyTheme()
}

const settingsPanel = ref<HTMLElement | null>(null)
let lastFocusedElement: HTMLElement | null = null

function getFocusableElements(container: HTMLElement): HTMLElement[] {
	return Array.from(
		container.querySelectorAll<HTMLElement>(
			'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
		),
	).filter((element) => !element.hasAttribute('disabled'))
}

function handleKeyDown(event: KeyboardEvent) {
	if (event.key === 'Escape') {
		open.value = false
		return
	}

	// Tab 焦点圈定在弹窗内，避免焦点逃逸到背景页面
	if (event.key !== 'Tab' || !open.value) return

	const panel = settingsPanel.value
	if (!panel) return

	const focusable = getFocusableElements(panel)
	if (!focusable.length) return

	const first = focusable[0]
	const last = focusable[focusable.length - 1]

	if (event.shiftKey && document.activeElement === first) {
		event.preventDefault()
		last.focus()
	} else if (!event.shiftKey && document.activeElement === last) {
		event.preventDefault()
		first.focus()
	}
}

function handleExternalLink(event: MouseEvent) {
	if (!externalLinksNewTab.value || !(event.target instanceof Element)) return
	const anchor = event.target.closest<HTMLAnchorElement>('a[href]')
	if (!anchor) return

	const destination = new URL(anchor.href, window.location.href)
	if (destination.origin !== window.location.origin) {
		anchor.target = '_blank'
		anchor.rel = 'noopener'
	}
}

watch(open, (isOpen) => {
	if (!import.meta.client) return
	document.body.style.overflow = isOpen ? 'hidden' : ''

	if (isOpen) {
		// 记住打开前的焦点，关闭时还原
		lastFocusedElement = document.activeElement as HTMLElement | null
		nextTick(() => {
			const panel = settingsPanel.value
			if (!panel) return
			const focusable = getFocusableElements(panel)
			;(focusable[0] ?? panel).focus()
		})
	} else {
		lastFocusedElement?.focus()
		lastFocusedElement = null
	}
})

watch([advancedRendering, reduceMotion, externalLinksNewTab], applyRenderingPreferences)

onMounted(() => {
	systemThemeQuery = window.matchMedia('(prefers-color-scheme: dark)')
	systemTheme.value = systemThemeQuery.matches ? 'dark' : 'light'
	preferredTheme.value = (localStorage.getItem('wisp-theme') as Theme | null) ?? 'system'
	advancedRendering.value = localStorage.getItem('wisp-advanced-rendering') !== 'false'
	reduceMotion.value = localStorage.getItem('wisp-reduce-motion') === 'true'
	externalLinksNewTab.value = localStorage.getItem('wisp-external-links-new-tab') !== 'false'
	applyTheme()
	applyRenderingPreferences()
	systemThemeQuery.addEventListener('change', handleSystemTheme)
	window.addEventListener('keydown', handleKeyDown)
	window.addEventListener('click', handleExternalLink, true)
})

onBeforeUnmount(() => {
	document.body.style.overflow = ''
	systemThemeQuery?.removeEventListener('change', handleSystemTheme)
	window.removeEventListener('keydown', handleKeyDown)
	window.removeEventListener('click', handleExternalLink, true)
})
</script>

<template>
	<Teleport to="body">
		<Transition name="settings-modal">
			<div v-if="open" class="settings-backdrop" @click.self="open = false">
				<section
					ref="settingsPanel"
					class="settings-panel"
					role="dialog"
					aria-modal="true"
					aria-labelledby="settings-title"
				>
					<header class="settings-header">
						<h2 id="settings-title">
							<SettingsIcon aria-hidden="true" />
							{{ formatMessage(messages.title) }}
						</h2>
						<ButtonStyled circular type="transparent">
							<button :aria-label="formatMessage(messages.close)" @click="open = false">
								<XIcon aria-hidden="true" />
							</button>
						</ButtonStyled>
					</header>

					<div class="settings-body">
						<aside class="settings-sidebar">
							<nav :aria-label="formatMessage(messages.title)">
								<button
									:class="{ selected: activeTab === 'appearance' }"
									@click="activeTab = 'appearance'"
								>
									<PaintbrushIcon aria-hidden="true" />
									{{ formatMessage(messages.appearanceTitle) }}
								</button>
								<button
									:class="{ selected: activeTab === 'language' }"
									@click="activeTab = 'language'"
								>
									<LanguagesIcon aria-hidden="true" />
									{{ formatMessage(messages.languageTitle) }}
								</button>
							</nav>

							<div class="settings-brand flex items-center gap-3 mt-auto px-1 pt-4 text-[var(--color-secondary)] text-[0.8rem]">
								<img src="/wisp.png" alt="" />
								<div>
									<strong>WispLauncher</strong>
									<span>Website {{ new Date().getFullYear() }}</span>
								</div>
							</div>
						</aside>

						<div class="settings-main">
							<section v-if="activeTab === 'appearance'" class="settings-pane px-8 pl-6 pb-12">
								<div class="settings-section">
									<h3>{{ formatMessage(messages.themeTitle) }}</h3>
									<p>{{ formatMessage(messages.themeDescription) }}</p>
									<ThemeSelector
										:update-color-theme="updateColorTheme"
										:current-theme="preferredTheme"
										:theme-options="themeOptions"
										:system-theme-color="systemTheme"
									/>
								</div>

								<div class="settings-section mt-7 pt-6 border-t border-divider">
									<h3>{{ formatMessage(messages.interfaceTitle) }}</h3>
									<p>{{ formatMessage(messages.interfaceDescription) }}</p>
									<div class="flex flex-col mt-3">
										<div class="setting-row">
											<label for="advanced-rendering">
												<strong>{{ formatMessage(messages.advancedRenderingTitle) }}</strong>
												<span>{{ formatMessage(messages.advancedRenderingDescription) }}</span>
											</label>
											<Toggle id="advanced-rendering" v-model="advancedRendering" />
										</div>
										<div class="setting-row">
											<label for="reduce-motion">
												<strong>{{ formatMessage(messages.reduceMotionTitle) }}</strong>
												<span>{{ formatMessage(messages.reduceMotionDescription) }}</span>
											</label>
											<Toggle id="reduce-motion" v-model="reduceMotion" />
										</div>
										<div class="setting-row">
											<label for="external-links">
												<strong>{{ formatMessage(messages.externalLinksTitle) }}</strong>
												<span>{{ formatMessage(messages.externalLinksDescription) }}</span>
											</label>
											<Toggle id="external-links" v-model="externalLinksNewTab" />
										</div>
									</div>
								</div>
							</section>

							<section v-else-if="activeTab === 'language'" class="settings-pane px-8 pl-6 pb-12 language-pane">
								<div class="settings-section">
									<h3>{{ formatMessage(messages.languageTitle) }}</h3>
									<p>{{ formatMessage(messages.languageDescription) }}</p>
									<LanguageSelector
										:current-locale="locale"
										:locales="supportedLocales"
										:on-locale-change="updateLocale"
										:is-changing="changingLocale"
									/>
								</div>
							</section>
						</div>
					</div>
				</section>
			</div>
		</Transition>
	</Teleport>
</template>

<style scoped lang="scss">
.settings-backdrop {
	position: fixed;
	inset: 0;
	z-index: 100;
	display: flex;
	align-items: center;
	justify-content: center;
	padding: 1rem;
	background: rgb(10 12 18 / 58%);
	backdrop-filter: blur(8px) saturate(90%);
}

.settings-panel {
	display: flex;
	flex-direction: column;
	width: min(60rem, 100%);
	height: min(40rem, calc(100vh - 2rem));
	overflow: hidden;
	border: 1px solid var(--color-divider);
	border-radius: 1.25rem;
	background: var(--color-raised-bg);
	box-shadow: 0 2rem 6rem rgb(0 0 0 / 42%);
}

.settings-header {
	display: flex;
	flex: 0 0 auto;
	align-items: center;
	justify-content: space-between;
	min-height: 5.25rem;
	padding: 0 1.5rem;
	border-bottom: 1px solid var(--color-divider);
	background: var(--color-raised-bg);

	h2 {
		display: flex;
		align-items: center;
		gap: 0.625rem;
		margin: 0;
		color: var(--color-contrast);
		font-size: 1.125rem;
		font-weight: 800;
	}

	svg {
		width: 1.125rem;
		height: 1.125rem;
	}
}

.settings-body {
	display: grid;
	grid-template-columns: 15.5rem minmax(0, 1fr);
	min-height: 0;
	flex: 1;
	padding: 1.5rem 0 0 1.5rem;
}

.settings-sidebar {
	display: flex;
	min-height: 0;
	flex-direction: column;
	padding: 0 1rem 0.75rem 0;
	border-right: 1px solid var(--color-divider);

	nav {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	button {
		display: flex;
		align-items: center;
		gap: 0.625rem;
		width: 100%;
		padding: 0.55rem 1rem;
		border: 0;
		border-radius: 0.75rem;
		background: transparent;
		color: var(--color-base);
		font: inherit;
		font-weight: 650;
		text-align: left;
		transition: 120ms ease;

		&:hover {
			background: var(--color-button-bg);
			color: var(--color-contrast);
			cursor: pointer;
		}

		&.selected {
			background: var(--color-brand);
			color: var(--color-brand-inverted);
			box-shadow: 0 0.5rem 1.5rem color-mix(in srgb, var(--color-brand) 16%, transparent);
		}

		svg {
			width: 1rem;
			height: 1rem;
			flex: 0 0 auto;
		}
	}
}

.settings-brand {
	img {
		width: 2.25rem;
		height: 2.25rem;
		object-fit: contain;
	}

	div {
		display: flex;
		min-width: 0;
		flex-direction: column;
		gap: 0.15rem;
	}

	strong {
		color: var(--color-base);
		font-size: 0.875rem;
	}

	span {
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}
}

.settings-main {
	position: relative;
	min-width: 0;
	min-height: 0;
	overflow-y: auto;
	scrollbar-color: var(--color-scrollbar) transparent;
}

.settings-main::after {
	position: sticky;
	bottom: 0;
	display: block;
	height: 2.5rem;
	margin-top: -2.5rem;
	background: linear-gradient(transparent, var(--color-raised-bg));
	content: '';
	pointer-events: none;
}

.settings-section {
	h3,
	p {
		margin: 0;
	}

	h3 {
		color: var(--color-contrast);
		font-size: 1.125rem;
		font-weight: 700;
	}

	p {
		margin-top: 0.25rem;
		color: var(--color-base);
	}
}

.setting-row {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 1.5rem;
	padding: 1rem 0;
	border-bottom: 1px solid color-mix(in srgb, var(--color-divider) 68%, transparent);

	label {
		display: flex;
		min-width: 0;
		flex: 1;
		flex-direction: column;
		gap: 0.25rem;
	}

	strong {
		color: var(--color-contrast);
		font-size: 0.95rem;
	}

	span {
		color: var(--color-secondary);
		font-size: 0.85rem;
		line-height: 1.45;
	}
}

.language-pane {
	:deep(.flex.flex-col.gap-4) {
		margin-top: 1rem;
	}
}

.settings-modal-enter-active,
.settings-modal-leave-active {
	transition: opacity 180ms ease;

	.settings-panel {
		transition: transform 180ms ease;
	}
}

.settings-modal-enter-from,
.settings-modal-leave-to {
	opacity: 0;

	.settings-panel {
		transform: translateY(0.75rem) scale(0.985);
	}
}

@media (max-width: 700px) {
	.settings-backdrop {
		align-items: flex-end;
		padding: 0;
	}

	.settings-panel {
		height: 94vh;
		border-radius: 1.25rem 1.25rem 0 0;
	}

	.settings-header {
		min-height: 4.5rem;
		padding: 0 1rem;
	}

	.settings-body {
		display: flex;
		flex-direction: column;
		padding: 0;
	}

	.settings-sidebar {
		padding: 0.75rem 1rem;
		border-right: 0;
		border-bottom: 1px solid var(--color-divider);

		nav {
			flex-direction: row;
		}

		button {
			justify-content: center;
		}
	}

	.settings-brand {
		display: none;
	}

	.settings-pane {
		padding: 1.25rem 1rem 3rem;
	}

	.setting-row {
		align-items: flex-start;
	}
}
</style>
