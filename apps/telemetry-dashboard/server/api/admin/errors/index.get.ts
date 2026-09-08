import { defineAdminHandler } from '../../../utils/handler'
import { getAdminApi } from '../../../utils/service'
import { getQueryRecord, parseErrorQuery } from '../../../utils/validation'

export default defineAdminHandler(async (event) =>
	getAdminApi(event).errors(parseErrorQuery(getQueryRecord(event))),
)
