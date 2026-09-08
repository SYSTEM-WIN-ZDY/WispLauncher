export interface BrowseReturnSnapshot<T> {
	url: string
	scrollTop: number
	state: T
}

let pendingSnapshot: BrowseReturnSnapshot<unknown> | null = null
let pendingReturnUrl: string | null = null

export function saveBrowseReturnSnapshot<T>(snapshot: BrowseReturnSnapshot<T>): void {
	pendingSnapshot = snapshot
}

export function consumeBrowseReturnSnapshot<T>(url: string): BrowseReturnSnapshot<T> | null {
	if (pendingReturnUrl !== url || pendingSnapshot?.url !== url) return null

	const snapshot = pendingSnapshot as BrowseReturnSnapshot<T>
	pendingSnapshot = null
	return snapshot
}

export function hasBrowseReturnSnapshot(url: string): boolean {
	return pendingSnapshot?.url === url
}

export function clearBrowseReturnSnapshot(): void {
	pendingSnapshot = null
	pendingReturnUrl = null
}

export function isBrowseReturnSourcePath(path: string): boolean {
	return path === '/downloads' || path.startsWith('/project/') || path.startsWith('/instance/')
}

export function prepareBrowseReturnNavigation(url: string, sourcePath: string): boolean {
	if (isBrowseReturnSourcePath(sourcePath) && hasBrowseReturnSnapshot(url)) {
		pendingReturnUrl = url
		return true
	}

	clearBrowseReturnSnapshot()
	return false
}

export function isBrowseReturnNavigation(url: string): boolean {
	return pendingReturnUrl === url
}

export function completeBrowseReturnNavigation(url: string): void {
	if (pendingReturnUrl === url) pendingReturnUrl = null
}
