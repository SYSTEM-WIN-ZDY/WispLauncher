import { invoke } from '@tauri-apps/api/core'
import type { App } from 'vue'

export type FrontendErrorReport = {
	error_type: string
	message: string
	stack?: string
	route?: string
	command?: string
	context?: string
}

function errorDetails(error: unknown): Pick<FrontendErrorReport, 'message' | 'stack'> {
	if (error instanceof Error) {
		return {
			message: error.message || error.name,
			stack: error.stack,
		}
	}
	if (typeof error === 'string') return { message: error }

	try {
		return { message: JSON.stringify(error) }
	} catch {
		return { message: String(error) }
	}
}

export function reportTelemetryError(
	error: unknown,
	options: Partial<Omit<FrontendErrorReport, 'message' | 'stack'>> = {},
): void {
	const details = errorDetails(error)
	void invoke('plugin:telemetry|submit_frontend_error', {
		report: {
			error_type: options.error_type ?? 'frontend',
			message: details.message,
			stack: details.stack,
			route: options.route ?? (window.location.hash || window.location.pathname),
			command: options.command,
			context: options.context,
		} satisfies FrontendErrorReport,
	}).catch(() => undefined)
}

export function installTelemetryErrorHandlers(app: App): void {
	app.config.errorHandler = (error, _instance, info) => {
		reportTelemetryError(error, {
			error_type: 'vue',
			context: info,
		})
		console.error(error)
	}

	window.addEventListener('error', (event) => {
		reportTelemetryError(event.error ?? event.message, {
			error_type: 'window_error',
			context: `${event.filename}:${event.lineno}:${event.colno}`,
		})
	})

	window.addEventListener('unhandledrejection', (event) => {
		reportTelemetryError(event.reason, { error_type: 'unhandled_rejection' })
	})

	window.addEventListener('online', () => {
		void invoke('plugin:telemetry|notify_online').catch(() => undefined)
	})
}
