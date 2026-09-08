import { createError, getRouterParam } from 'h3'

import { defineAdminHandler } from '../../../utils/handler'
import { getAdminApi } from '../../../utils/service'

export default defineAdminHandler(async (event) => {
	const fingerprint = getRouterParam(event, 'fingerprint') ?? ''
	if (!fingerprint || fingerprint.length > 128) throw createError({ statusCode: 404 })
	const detail = await getAdminApi(event).errorDetail(fingerprint)
	if (!detail) throw createError({ statusCode: 404, statusMessage: 'Error group not found' })
	return detail
})
