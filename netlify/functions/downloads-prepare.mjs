// 为Miawa下载源获取直链
// 成功时 302 到直链，失败时降级到无token页面
const MIAWA_API = 'https://miawa.cn/api/v2/downloads/prepare'
const MIAWA_DOWNLOAD_BASE = 'https://miawa.cn/download'

function encodeFilePath(filePath) {
	return filePath
		.split('/')
		.map((segment) => encodeURIComponent(segment))
		.join('/')
}

function redirect(location) {
	return new Response(null, {
		status: 302,
		headers: {
			Location: location,
			'Cache-Control': 'no-store',
		},
	})
}

function isMiawaUrl(value) {
	try {
		const url = new URL(value, 'https://miawa.cn')
		return url.hostname === 'miawa.cn' || url.hostname.endsWith('.miawa.cn')
	} catch {
		return false
	}
}

function toAbsoluteMiawaUrl(downloadPath) {
	if (/^https?:\/\//i.test(downloadPath)) {
		return new URL(downloadPath).toString()
	}
	if (downloadPath.startsWith('//')) {
		return new URL(`https:${downloadPath}`).toString()
	}
	return new URL(downloadPath, 'https://miawa.cn').toString()
}

function getFilePath(request) {
	try {
		if (request.url) {
			const filePath = new URL(request.url).searchParams.get('file_path')
			if (filePath) return filePath
		}
	} catch {}
	return request.queryStringParameters?.file_path ?? null
}

export default async (request) => {
	try {
		const filePath = getFilePath(request)
		if (!filePath) {
			return new Response('Missing file_path', { status: 400 })
		}

		const response = await fetch(MIAWA_API, {
			method: 'POST',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json',
				'User-Agent': ' Wisp-Website',
			},
			body: JSON.stringify({ file_path: filePath }),
			signal: AbortSignal.timeout(8000),
		})

		if (!response.ok) {
			return redirect(`${MIAWA_DOWNLOAD_BASE}/${encodeFilePath(filePath)}`)
		}

		const payload = await response.json()
		const downloadPath = payload?.data?.download_url
		if (typeof downloadPath !== 'string' || !downloadPath) {
			return redirect(`${MIAWA_DOWNLOAD_BASE}/${encodeFilePath(filePath)}`)
		}

		const location = isMiawaUrl(downloadPath)
			? toAbsoluteMiawaUrl(downloadPath)
			: `${MIAWA_DOWNLOAD_BASE}/${encodeFilePath(filePath)}`
		return redirect(location)
	} catch {
		const filePath = getFilePath(request)
		if (!filePath) {
			return new Response('Missing file_path', { status: 400 })
		}
		return redirect(`${MIAWA_DOWNLOAD_BASE}/${encodeFilePath(filePath)}`)
	}
}
