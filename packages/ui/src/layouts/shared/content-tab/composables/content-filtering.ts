import type { ClientWarningType, ContentItem } from '../types'

const CLIENT_ONLY_ENVIRONMENTS = new Set(['client_only', 'singleplayer_only'])

export function isClientOnlyEnvironment(env?: string | null): boolean {
	return !!env && CLIENT_ONLY_ENVIRONMENTS.has(env)
}

export function getClientWarningType(item: ContentItem): ClientWarningType | null {
	if (item.pack_client_retained) return 'retained'
	if (item.pack_client_depends) return 'depends'
	if (isClientOnlyEnvironment(item.environment)) return 'environment'
	return null
}

export function isPresentContentItem(item: ContentItem): boolean {
	return (
		item.instanceMaterializationState == null || item.instanceMaterializationState === 'present'
	)
}

export function isEnabledContentItem(item: ContentItem): boolean {
	return isPresentContentItem(item) && item.enabled === true
}

export function isDisabledContentItem(item: ContentItem): boolean {
	return isPresentContentItem(item) && item.enabled === false
}

export function canToggleContentItem(item: ContentItem): boolean {
	return (
		isPresentContentItem(item) &&
		item.enabled !== undefined &&
		item.instanceCapabilities?.canToggle !== false
	)
}

export interface ContentFilterOption {
	id: string
	label: string
}
