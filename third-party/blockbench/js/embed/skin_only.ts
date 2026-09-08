import { format } from '../formats/minecraft/skin'
import { BarMenu, MenuBar } from '../interface/menu_bar'
import { resizeWindow } from '../interface/interface'
import { setActivePanel } from '../interface/panels'
import { setStartScreen } from '../io/project'
import { onVueSetup } from '../util/util'

const axolotlThemeColors = [
	'ui',
	'back',
	'dark',
	'border',
	'selected',
	'elevated',
	'button',
	'bright_ui',
	'accent',
	'accent_highlight',
	'focus_ring',
	'hover',
	'frame',
	'text',
	'light',
	'accent_text',
	'bright_ui_text',
	'subtle_text',
	'grid',
	'wireframe',
	'checkerboard',
	'menu_separator',
	'bright_border',
] as const

const axolotlThemeMetrics = [
	'gap-xs',
	'gap-sm',
	'gap-md',
	'gap-lg',
	'radius-xs',
	'radius-sm',
	'radius-md',
] as const

type AxolotlThemeMessage = {
	type: 'axolotl-skin-theme'
	theme: {
		dark: boolean
		colors: Record<string, string>
		metrics: Record<string, string>
	}
}

function applyAxolotlTheme(event: MessageEvent<unknown>): void {
	if (event.source !== window.parent || !event.data || typeof event.data !== 'object') return
	const message = event.data as Partial<AxolotlThemeMessage>
	if (
		message.type !== 'axolotl-skin-theme' ||
		!message.theme ||
		typeof message.theme.dark !== 'boolean' ||
		!message.theme.colors ||
		!message.theme.metrics
	)
		return

	for (const color of axolotlThemeColors) {
		const value = message.theme.colors[color]
		if (typeof value === 'string' && value)
			document.body.style.setProperty(`--color-${color}`, value)
	}
	for (const metric of axolotlThemeMetrics) {
		const value = message.theme.metrics[metric]
		if (typeof value === 'string' && value)
			document.body.style.setProperty(`--axolotl-${metric}`, value)
	}
	document.body.classList.toggle('axolotl_dark_theme', message.theme.dark)
	document.body.classList.toggle('axolotl_light_theme', !message.theme.dark)
	document.documentElement.style.colorScheme = message.theme.dark ? 'dark' : 'light'
}

/**
 * Restricts the existing Blockbench workbench to Minecraft player skin work.
 * The painter, UV editor, preview, and undo stack remain Blockbench's native
 * implementations; only unrelated format and workspace entry points are removed.
 */
export function isSkinOnlyEmbed(): boolean {
	return Blockbench.queries?.embed === 'skin'
}

export function setupSkinOnlyEmbed(): void {
	if (!isSkinOnlyEmbed()) return

	document.body.classList.add('skin_only_embed')
	window.addEventListener('message', applyAxolotlTheme)
	document.getElementById('skin_only_attribution').hidden = false
	for (const id of Object.keys(MenuBar.menus)) {
		MenuBar.menus[id].delete()
	}

	new BarMenu('file', [
		{
			id: 'new_minecraft_skin',
			name: 'dialog.skin.title',
			icon: 'icon-player',
			click: () => format.new(),
		},
		new MenuSeparator('export'),
		'export_minecraft_skin',
	])
	new BarMenu('edit', ['undo', 'redo'])
	new BarMenu('skin', [
		'custom_skin_poses',
		'add_custom_skin_pose',
		new MenuSeparator('edit'),
		'toggle_skin_layer',
		'explode_skin_model',
		'convert_minecraft_skin_variant',
	])
	new BarMenu('image', [
		'adjust_brightness_contrast',
		'adjust_saturation_hue',
		'adjust_opacity',
		'invert_colors',
		new MenuSeparator('transform'),
		'flip_texture_x',
		'flip_texture_y',
		'rotate_texture_cw',
		'rotate_texture_ccw',
	])
	MenuBar.update()
	const onConfirm = format.setup_dialog.onConfirm
	let unlockInitialDialog: (() => void) | undefined
	format.setup_dialog.onConfirm = function (...args) {
		const result = onConfirm.apply(this, args)
		if (result !== false) {
			unlockInitialDialog?.()
			unlockInitialDialog = undefined
			window.requestAnimationFrame(() => {
				format.setup_dialog.hide()
				setActivePanel('uv')
				resizeWindow()
			})
		}
		return result
	}

	function openRequiredInitialDialog() {
		const originalCancel = format.setup_dialog.cancel.bind(format.setup_dialog)
		const originalCancelOnClickOutside = format.setup_dialog.cancel_on_click_outside
		format.setup_dialog.cancel_on_click_outside = false
		format.setup_dialog.cancel = () => {}
		unlockInitialDialog = () => {
			format.setup_dialog.cancel = originalCancel
			format.setup_dialog.cancel_on_click_outside = originalCancelOnClickOutside
		}
		format.new()
		window.requestAnimationFrame(() => {
			format.setup_dialog.object?.querySelector('.cancel_btn')?.remove()
			format.setup_dialog.object?.querySelector('.dialog_close_button')?.remove()
		})
	}

	Blockbench.skinOnlyEmbed = { openRequiredInitialDialog }
}

onVueSetup(() => {
	if (!isSkinOnlyEmbed()) return
	setStartScreen(false)
	window.requestAnimationFrame(() => {
		resizeWindow()
		Blockbench.skinOnlyEmbed.openRequiredInitialDialog()
		window.parent.postMessage({ type: 'axolotl-skin-theme-ready' }, '*')
	})
})
