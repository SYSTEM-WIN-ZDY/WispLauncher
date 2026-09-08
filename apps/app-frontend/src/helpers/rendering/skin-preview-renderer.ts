import { ClassicPlayerModel, SlimPlayerModel } from '@modrinth/assets'
import {
	applyCapeTexture,
	createTransparentTexture,
	disposeCaches,
	setupSkinModel,
} from '@modrinth/ui/src/utils/webgl/skin-rendering'
import * as THREE from 'three'

import type { Cape, Skin } from '../skins'
import { determineModelType, get_normalized_skin_texture } from '../skins'
import { headStorage } from '../storage/head-storage'
import { skinPreviewStorage } from '../storage/skin-preview-storage'
import {
	blobToDataUrl,
	cleanupUnusedPreviews,
	generateHeadRender,
	getHeadRenderKey,
	headBlobUrlMap,
	type RawRenderResult,
	type RenderResult,
	skinBlobUrlMap,
} from './batch-skin-renderer'

class BatchSkinRenderer {
	private renderer: THREE.WebGLRenderer | null = null
	private scene: THREE.Scene | null = null
	private camera: THREE.PerspectiveCamera | null = null
	private currentModel: THREE.Group | null = null
	private transparentTexture: THREE.Texture | null = null
	private readonly width: number
	private readonly height: number

	constructor(width: number = 360, height: number = 504) {
		this.width = width
		this.height = height
	}

	private initializeRenderer(): void {
		if (this.renderer) return

		const canvas = document.createElement('canvas')
		canvas.width = this.width
		canvas.height = this.height

		// Some WebKitGTK builds hang when creating a WebGL2 context with
		// `preserveDrawingBuffer: true`. Omit it and capture the frame
		// synchronously via toDataURL right after render() instead.
		this.renderer = new THREE.WebGLRenderer({
			canvas: canvas,
			antialias: true,
			alpha: true,
		})

		this.renderer.outputColorSpace = THREE.SRGBColorSpace
		this.renderer.shadowMap.enabled = false
		this.renderer.toneMapping = THREE.NoToneMapping
		this.renderer.toneMappingExposure = 10.0
		this.renderer.setClearColor(0x000000, 0)
		this.renderer.setSize(this.width, this.height)

		this.scene = new THREE.Scene()
		this.camera = new THREE.PerspectiveCamera(20, this.width / this.height, 0.4, 1000)

		const ambientLight = new THREE.AmbientLight(0xffffff, 2)
		const directionalLight = new THREE.DirectionalLight(0xffffff, 1.2)
		directionalLight.castShadow = false
		directionalLight.position.set(2, 4, 3)
		this.scene.add(ambientLight)
		this.scene.add(directionalLight)
	}

	public async renderSkin(
		textureUrl: string,
		modelUrl: string,
		capeUrl?: string,
	): Promise<RawRenderResult> {
		this.initializeRenderer()

		this.clearScene()

		await this.setupModel(modelUrl, textureUrl, capeUrl)

		const headPart = this.currentModel!.getObjectByName('Head')
		let lookAtTarget: [number, number, number]

		if (headPart) {
			const headPosition = new THREE.Vector3()
			headPart.getWorldPosition(headPosition)
			lookAtTarget = [headPosition.x, headPosition.y - 0.3, headPosition.z]
		} else {
			throw new Error("Failed to find 'Head' object in model.")
		}

		const frontCameraPos: [number, number, number] = [-1.3, 1, 6.3]
		const forwards = await this.renderView(frontCameraPos, lookAtTarget)

		return { forwards }
	}

	private async renderView(
		cameraPosition: [number, number, number],
		lookAtPosition: [number, number, number],
	): Promise<Blob> {
		if (!this.camera || !this.renderer || !this.scene) {
			throw new Error('Renderer not initialized')
		}

		this.camera.position.set(...cameraPosition)
		this.camera.lookAt(...lookAtPosition)

		this.renderer.render(this.scene, this.camera)

		// Without `preserveDrawingBuffer` the buffer is cleared after the
		// compositor runs, so capture synchronously right after render().
		// toDataURL is synchronous (unlike toBlob) and cannot hang.
		const dataUrl = this.renderer.domElement.toDataURL('image/png')
		const base64 = dataUrl.split(',')[1]
		const binary = atob(base64)
		const bytes = new Uint8Array(binary.length)
		for (let i = 0; i < binary.length; i++) {
			bytes[i] = binary.charCodeAt(i)
		}
		return new Blob([bytes], { type: 'image/png' })
	}

	private async setupModel(modelUrl: string, textureUrl: string, capeUrl?: string): Promise<void> {
		if (!this.scene) {
			throw new Error('Renderer not initialized')
		}

		const { model } = await setupSkinModel(modelUrl, textureUrl, capeUrl)

		if (!capeUrl) {
			applyCapeTexture(model, null, this.getTransparentTexture())
		}

		const group = new THREE.Group()
		group.add(model)
		group.position.set(0, 0.3, 1.95)
		group.scale.set(0.8, 0.8, 0.8)

		this.scene.add(group)
		this.currentModel = group
	}

	private getTransparentTexture(): THREE.Texture {
		if (!this.transparentTexture) {
			this.transparentTexture = createTransparentTexture()
		}

		return this.transparentTexture
	}

	private clearScene(): void {
		if (!this.scene || !this.currentModel) return

		this.scene.remove(this.currentModel)
		this.currentModel.clear()
		this.currentModel = null
	}

	public dispose(): void {
		this.clearScene()

		if (this.transparentTexture) {
			this.transparentTexture.dispose()
			this.transparentTexture = null
		}

		if (this.renderer) {
			this.renderer.dispose()
		}

		this.renderer = null
		this.scene = null
		this.camera = null

		disposeCaches()
	}
}

function getModelUrlForVariant(variant: string): string {
	switch (variant) {
		case 'SLIM':
			return SlimPlayerModel
		case 'CLASSIC':
		case 'UNKNOWN':
		default:
			return ClassicPlayerModel
	}
}

const DEBUG_MODE = false

let sharedRenderer: BatchSkinRenderer | null = null
let latestPreviewGeneration = 0
let previewGenerationQueue: Promise<void> = Promise.resolve()

function getSharedRenderer(): BatchSkinRenderer {
	if (!sharedRenderer) {
		sharedRenderer = new BatchSkinRenderer()
	}
	return sharedRenderer
}

export function disposeSharedRenderer(): void {
	if (sharedRenderer) {
		sharedRenderer.dispose()
		sharedRenderer = null
	}
}

export function generateSkinPreviews(skins: Skin[], capes: Cape[]): Promise<void> {
	const generation = ++latestPreviewGeneration
	const skinsSnapshot = [...skins]
	const capesSnapshot = [...capes]

	const generationPromise = previewGenerationQueue.then(() =>
		generateSkinPreviewsForGeneration(skinsSnapshot, capesSnapshot, generation),
	)

	previewGenerationQueue = generationPromise.catch(() => {})

	return generationPromise
}

async function generateSkinPreviewsForGeneration(
	skins: Skin[],
	capes: Cape[],
	generation: number,
): Promise<void> {
	const isCurrentGeneration = () => generation === latestPreviewGeneration

	let cachedSkinPreviews: Record<string, RawRenderResult | null> = {}
	let cachedHeadPreviews: Record<string, Blob | null> = {}
	const skinKeys = skins.map(
		(skin) => `${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`,
	)
	const headKeys = skins.map((skin) => getHeadRenderKey(skin.texture_key))

	try {
		// Some WebKitGTK builds hang when reading IndexedDB. Guard the cache
		// read with a timeout and fall through to render without cache.
		;[cachedSkinPreviews, cachedHeadPreviews] = await Promise.race([
			Promise.all([
				skinPreviewStorage.batchRetrieve(skinKeys),
				headStorage.batchRetrieve(headKeys),
			]),
			new Promise<never>((_, reject) =>
				setTimeout(() => reject(new Error('skin preview cache read timed out')), 3000),
			),
		])
	} catch (error) {
		console.warn('Failed to read skin preview cache, rendering without cache:', error)
	}

	if (!isCurrentGeneration()) return

	try {
		for (let i = 0; i < skins.length; i++) {
			const skinKey = skinKeys[i]
			const headKey = headKeys[i]

			const rawCached = cachedSkinPreviews[skinKey]
			if (rawCached && !skinBlobUrlMap.has(skinKey)) {
				const cached: RenderResult = {
					forwards: URL.createObjectURL(rawCached.forwards),
				}
				skinBlobUrlMap.set(skinKey, cached)
			}

			const cachedHead = cachedHeadPreviews[headKey]
			if (cachedHead && !headBlobUrlMap.has(headKey)) {
				blobToDataUrl(cachedHead)
					.then((dataUrl) => {
						if (!headBlobUrlMap.has(headKey)) {
							headBlobUrlMap.set(headKey, dataUrl)
						}
					})
					.catch((error) => {
						console.warn('Failed to convert cached head render to data URL:', error)
					})
			}
		}

		for (const skin of skins) {
			if (!isCurrentGeneration()) return

			const key = `${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`

			if (skinBlobUrlMap.has(key)) {
				if (DEBUG_MODE) {
					const result = skinBlobUrlMap.get(key)!
					URL.revokeObjectURL(result.forwards)
					skinBlobUrlMap.delete(key)
				} else continue
			}

			const renderer = getSharedRenderer()

			let variant = skin.variant
			if (variant === 'UNKNOWN') {
				try {
					variant = await determineModelType(skin.texture)
				} catch (error) {
					console.error(`Failed to determine model type for skin ${key}:`, error)
					variant = 'CLASSIC'
				}
			}

			const modelUrl = getModelUrlForVariant(variant)
			const cape: Cape | undefined = capes.find((_cape) => _cape.id === skin.cape_id)
			const rawRenderResult = await renderer.renderSkin(
				await get_normalized_skin_texture(skin),
				modelUrl,
				cape?.texture,
			)

			if (!isCurrentGeneration()) return

			const renderResult: RenderResult = {
				forwards: URL.createObjectURL(rawRenderResult.forwards),
			}

			skinBlobUrlMap.set(key, renderResult)

			try {
				await skinPreviewStorage.store(key, rawRenderResult)
			} catch (error) {
				console.warn('Failed to store skin preview in persistent storage:', error)
			}

			const headKey = getHeadRenderKey(skin.texture_key)
			if (!headBlobUrlMap.has(headKey)) {
				await generateHeadRender(skin)
			}
		}
	} finally {
		disposeSharedRenderer()

		if (isCurrentGeneration()) {
			try {
				await cleanupUnusedPreviews(skins)

				await skinPreviewStorage.debugCalculateStorage()
				await headStorage.debugCalculateStorage()
			} catch (error) {
				console.warn('Skin preview cleanup/debug failed:', error)
			}
		}
	}
}
