// Netlify Function：转发最新版本元数据。
// 客户端直连 CNB 会被 CORS 拦截（CNB 只放行 docs.cnb.cool），
// 由本站服务器代为拉取（服务器无 CORS 限制），每次请求实时转发。
const CNB_URL =
	'https://cnb.cool/axlmc/ Wisp/-/git/raw/main/apps/website/releases/latest.json'

export default async () => {
	try {
		const response = await fetch(CNB_URL, {
			headers: { 'User-Agent': ' Wisp-Website' },
			signal: AbortSignal.timeout(8000),
		})
		if (!response.ok) {
			return new Response(`CNB source returned ${response.status}`, { status: 502 })
		}
		return new Response(await response.text(), {
			headers: {
				'Content-Type': 'application/json',
				// 浏览器不缓存（每次访问实时请求），Netlify CDN 边缘缓存 5 分钟
				// 以节省 Function 调用额度；错误响应不缓存。
				'Cache-Control': 'public, max-age=0, s-maxage=300',
			},
		})
	} catch (error) {
		return new Response(`Failed to fetch release metadata: ${error.message}`, { status: 502 })
	}
}
