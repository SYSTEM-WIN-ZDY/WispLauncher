import { defineAdminHandler } from '../../utils/handler'
import { getAdminApi } from '../../utils/service'

export default defineAdminHandler(async (event) => getAdminApi(event).system())
