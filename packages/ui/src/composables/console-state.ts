import { type Ref, shallowRef, triggerRef } from 'vue'

import { detectLogLevel } from '../layouts/shared/console/composables/log-level'
import type { Log4jEvent, LogLevel, LogLine } from '../layouts/shared/console/types'

const ARCHIVE_CAPACITY = 500_000
const BATCH_TIMEOUT_MS = 300
const INITIAL_BATCH_SIZE = 256
const ENTRY_START_RE = /^\[\d{2}:\d{2}:\d{2}\]/

export interface ConsoleState {
	output: Ref<LogLine[]>
	addLog4jEvent: (event: Log4jEvent) => void
	addLegacyLog: (message: string) => Promise<void>
	clear: () => void
}

function groupContinuations(lines: LogLine[]): LogLine[] {
	if (lines.length <= 1) return lines

	const groups: LogLine[][] = []

	for (const line of lines) {
		if (ENTRY_START_RE.test(line.text)) {
			groups.push([line])
		} else if (groups.length > 0) {
			let target = groups.length - 1
			const lastEntry = groups[target][0]

			if (lastEntry.level !== 'error' && lastEntry.level !== 'warn') {
				if (line.level === 'error' || line.level === null) {
					for (let i = groups.length - 2; i >= 0; i--) {
						if (groups[i][0].level === 'error' || groups[i][0].level === 'warn') {
							target = i
							break
						}
					}
				}
			}

			groups[target].push(line)
		} else {
			groups.push([line])
		}
	}

	return groups.flat()
}

function mapLog4jLevel(level?: string): LogLevel | null {
	if (!level) return null
	switch (level.toUpperCase()) {
		case 'FATAL':
		case 'ERROR':
			return 'error'
		case 'WARN':
			return 'warn'
		case 'INFO':
			return 'info'
		case 'DEBUG':
			return 'debug'
		case 'TRACE':
			return 'trace'
		default:
			return null
	}
}

function formatTimestamp(millis?: number): string {
	if (!millis) return ''
	const date = new Date(millis)
	const hours = String(date.getHours()).padStart(2, '0')
	const minutes = String(date.getMinutes()).padStart(2, '0')
	const seconds = String(date.getSeconds()).padStart(2, '0')
	return `[${hours}:${minutes}:${seconds}]`
}

function formatLog4jLines(event: Log4jEvent): LogLine[] {
	const level = mapLog4jLevel(event.level)
	const time = formatTimestamp(event.timestamp_millis)
	const thread = event.thread_name ?? ''
	const levelText = event.level ?? ''
	const message = event.message?.trim() ?? ''
	const prefix = time ? `${time} [${thread}/${levelText}]: ` : `[${thread}/${levelText}]: `
	const messageLines = message.split(/[\r\n]+/)
	const lines: LogLine[] = [{ text: prefix + messageLines[0], level }]

	for (let i = 1; i < messageLines.length; i++) {
		if (!messageLines[i]) continue
		lines.push({ text: messageLines[i], level })
	}

	if (event.throwable) {
		for (const line of event.throwable.split(/[\r\n]+/)) {
			if (!line) continue
			lines.push({ text: line, level: 'error' })
		}
	}

	return lines
}

function textToLogLine(text: string): LogLine {
	return { text, level: detectLogLevel(text) }
}

export function createConsoleState(): ConsoleState {
	const output = shallowRef<LogLine[]>([])
	let lineBuffer: LogLine[] = []
	let batchTimer: ReturnType<typeof setTimeout> | null = null

	function flushBuffer() {
		if (lineBuffer.length === 0) return

		const lines = groupContinuations(lineBuffer)
		lineBuffer = []
		batchTimer = null
		for (const line of lines) {
			output.value.push(line)
		}

		const overflow = output.value.length - ARCHIVE_CAPACITY
		if (overflow > 0) {
			output.value.splice(0, overflow)
		}

		triggerRef(output)
	}

	function addLines(lines: LogLine[]) {
		if (lines.length === 0) return

		if (output.value.length === 0 && lines.length >= INITIAL_BATCH_SIZE) {
			lineBuffer = lines
			flushBuffer()
			return
		}

		for (const line of lines) {
			lineBuffer.push(line)
		}
		if (!batchTimer) {
			batchTimer = setTimeout(flushBuffer, BATCH_TIMEOUT_MS)
		}
	}

	function addLog4jEvent(event: Log4jEvent) {
		addLines(formatLog4jLines(event))
	}

	// 历史日志/缓冲大文本一次进入数据（保持行序），渲染由高亮管线分帧消化，
	// 避免分块写入把实时事件插到历史行之间造成顺序错乱
	function addLegacyLog(message: string): Promise<void> {
		const lines = message
			.split(/[\r\n]+/)
			.filter(Boolean)
			.map(textToLogLine)

		let parentLevel: LogLevel | null = null
		for (const line of lines) {
			if (ENTRY_START_RE.test(line.text)) {
				parentLevel = line.level
			} else if (line.level === null && parentLevel !== null) {
				line.level = parentLevel
			}
		}

		for (const line of lines) {
			output.value.push(line)
		}
		const overflow = output.value.length - ARCHIVE_CAPACITY
		if (overflow > 0) {
			output.value.splice(0, overflow)
		}
		triggerRef(output)
		return Promise.resolve()
	}

	function clear() {
		output.value = []
		lineBuffer = []
		if (batchTimer) {
			clearTimeout(batchTimer)
			batchTimer = null
		}
	}

	return {
		output,
		addLog4jEvent,
		addLegacyLog,
		clear,
	}
}
