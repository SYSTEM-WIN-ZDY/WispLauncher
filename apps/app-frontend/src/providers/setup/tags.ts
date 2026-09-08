import type { AbstractWebNotificationManager } from '@modrinth/ui'
import { provideTags } from '@modrinth/ui'
import { ref } from 'vue'

import { get_game_versions, get_loaders } from '@/helpers/tags'

export function setupTagsProvider(
	notificationManager: AbstractWebNotificationManager,
	stateInitialization: Promise<void>,
) {
	const { handleError } = notificationManager

	const gameVersions = ref([])
	const loaders = ref([])
	stateInitialization
		.then(() => {
			get_game_versions()
				.then((v) => {
					gameVersions.value = v
				})
				.catch(handleError)
			get_loaders()
				.then((v) => {
					loaders.value = v
				})
				.catch(handleError)
		})
		.catch(() => {})

	provideTags({ gameVersions, loaders })
}
