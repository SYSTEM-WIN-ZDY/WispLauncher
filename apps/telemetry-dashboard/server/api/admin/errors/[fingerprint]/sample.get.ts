import { createError, getRouterParam } from 'h3'

import { defineAdminHandler } from '../../../../utils/handler'
import { getAdminApi } from '../../../../utils/service'

export default defineAdminHandler(async (event) => {
	const fingerprint = getRouterParam(event, 'fingerprint') ?? ''
	if (!fingerprint || fingerprint.length > 128) throw createError({ statusCode: 404 })
	const sample = await getAdminApi(event).errorSample(fingerprint)
	if (!sample)
		throw createError({ statusCode: 404, statusMessage: 'No registered sample is available' })
	return sample
})
