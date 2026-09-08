import { buildLocaleMessages, createMessageCompiler, type CrowdinMessages } from '@modrinth/ui'
import { uiLocaleModulesEager } from '@modrinth/ui/src/locales.eager.ts'
import { createI18n } from 'vue-i18n'

const localeModules = import.meta.glob<{ default: CrowdinMessages }>('./locales/*/index.json', {
	eager: true,
})

const i18n = createI18n({
	legacy: false,
	locale: 'en-US',
	fallbackLocale: 'en-US',
	messageCompiler: createMessageCompiler(),
	missingWarn: false,
	fallbackWarn: false,
	messages: buildLocaleMessages(localeModules, uiLocaleModulesEager),
})

export function resolveInitialLocale(preferredLocales: readonly string[]): string {
	const availableLocales = new Set(i18n.global.availableLocales)

	for (const preferredLocale of preferredLocales) {
		const normalizedLocale = preferredLocale.replace('_', '-')
		if (availableLocales.has(normalizedLocale)) return normalizedLocale

		const language = normalizedLocale.split('-')[0].toLowerCase()
		if (language === 'zh') {
			const traditionalChinese = /-(tw|hk|mo)|-hant/i.test(normalizedLocale)
			return traditionalChinese ? 'zh-TW' : 'zh-CN'
		}

		const languageMatch = i18n.global.availableLocales.find((locale) =>
			locale.toLowerCase().startsWith(`${language}-`),
		)
		if (languageMatch) return languageMatch
	}

	return 'en-US'
}

export default i18n
