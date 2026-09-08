//从MiawaAPI获取最新版本
const MIAWA_API = 'https://miawa.cn/api/v2/launchers'

export default async () => {
	try {
		const response = await fetch(MIAWA_API, {
			headers: {
				Accept: 'application/json',
				'User-Agent': ' Wisp-Website',
			},
			signal: AbortSignal.timeout(8000),
		})
		if (!response.ok) {
			return new Response(`Miawa API returned ${response.status}`, { status: 502 })
		}

		const payload = await response.json()
		const launcher = payload?.data?.wisp?.[0]
		if (!launcher?.tag_name || !Array.isArray(launcher.assets)) {
			return new Response('Invalid Miawa launcher payload', { status: 502 })
		}

		const normalized = {
			tag_name: launcher.tag_name,
			assets: launcher.assets
				.map((asset) => (typeof asset === 'string' ? asset : asset?.name))
				.filter((name) => typeof name === 'string'),
		}

		return new Response(JSON.stringify(normalized), {
			headers: {
				'Content-Type': 'application/json',
				'Cache-Control': 'public, max-age=0, s-maxage=300',
			},
		})
	} catch (error) {
		return new Response(`Failed to fetch Miawa releases: ${error.message}`, { status: 502 })
	}
}
