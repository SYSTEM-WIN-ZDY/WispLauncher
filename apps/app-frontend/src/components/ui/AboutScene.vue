<template>
	<canvas id="about_scene" class="size-full" />
</template>

<script setup lang="ts">
import { onMounted, onScopeDispose } from 'vue'
import * as THREE from 'three'

import allayTextureUrl from '@/assets/allay-texture.png'
import diamondTextureUrl from '@/assets/diamond-block.png'

const TEX_SIZE = 32
const MODEL_HEIGHT = 9
// Keep 0: the wings are vertical planes at ±45°+flap around y; a large pose yaw turns one
// wing edge-on to the camera (|sin|→0) and it visually disappears.
const POSE_YAW = 0

const HOVER_MIN_MS = 900
const HOVER_MAX_MS = 1600
const MOVE_DURATION_MS = 1400
const BOB_AMPLITUDE = 6
const BOB_FREQUENCY = 2.2
const EDGE_MARGIN = 20

const SIZE_MIN = 0.52
const SIZE_MAX = 1.0

const MOUSE_SPEED_BOOST = 0.6
const MOUSE_SPEED_SMOOTH = 0.08

// ─── Allay model: faithful port of bytecode (ModelPart$Cube / Polygon inverted from 26.2.jar) ───
// Cube(int du, int dv, minX, minY, minZ, sizeX, sizeY, sizeZ, inflateX/Y/Z, mirror, texW, texH)
// Polygon remap (bytes 23-111): vertex0→(u2/texW, v1/texH), vertex1→(u1/texW, v1/texH),
//                               vertex2→(u1/texW, v2/texH), vertex3→(u2/texW, v2/texH)

interface CubeDef {
	x: number
	y: number
	z: number
	w: number
	h: number
	d: number
	inflate: number
	du: number
	dv: number
}

interface PartDef {
	name: string
	parent: 'root' | 'body'
	offset: [number, number, number]
	cubes: CubeDef[]
}

const PARTS: PartDef[] = [
	{
		name: 'head',
		parent: 'root',
		offset: [0, -3.99, 0],
		cubes: [{ x: -2.5, y: -5, z: -2.5, w: 5, h: 5, d: 5, inflate: 0, du: 0, dv: 0 }],
	},
	{
		name: 'body',
		parent: 'root',
		offset: [0, -4, 0],
		cubes: [
			{ x: -1.5, y: 0, z: -1, w: 3, h: 4, d: 2, inflate: 0, du: 0, dv: 10 },
			{ x: -1.5, y: 0, z: -1, w: 3, h: 5, d: 2, inflate: -0.2, du: 0, dv: 16 },
		],
	},
	{
		name: 'right_arm',
		parent: 'body',
		offset: [-1.75, 0.5, 0],
		cubes: [{ x: -0.75, y: -0.5, z: -1, w: 1, h: 4, d: 2, inflate: -0.01, du: 23, dv: 0 }],
	},
	{
		name: 'left_arm',
		parent: 'body',
		offset: [1.75, 0.5, 0],
		cubes: [{ x: -0.25, y: -0.5, z: -1, w: 1, h: 4, d: 2, inflate: -0.01, du: 23, dv: 6 }],
	},
	{
		name: 'right_wing',
		parent: 'body',
		offset: [-0.5, 0, 0.6],
		cubes: [{ x: 0, y: 1, z: 0, w: 0, h: 5, d: 8, inflate: 0, du: 16, dv: 14 }],
	},
	{
		name: 'left_wing',
		parent: 'body',
		offset: [0.5, 0, 0.6],
		cubes: [{ x: 0, y: 1, z: 0, w: 0, h: 5, d: 8, inflate: 0, du: 16, dv: 14 }],
	},
]

// 8 cube corners (mirror=false everywhere in the allay), matching javap slot order 19…26.
// V0=(minX,minY,minZ) V1=(maxX,minY,minZ) V2=(maxX,maxY,minZ) V3=(minX,maxY,minZ)
// V4=(minX,minY,maxZ) V5=(maxX,minY,maxZ) V6=(maxX,maxY,maxZ) V7=(minX,maxY,maxZ)
// Inflate shrinks (min -= inflate, max += inflate), like CubeDeformation growth inverted.
// U-slot offsets from javap: U1=du U2=du+d U3=du+d+w U4=du+d+2w U5=du+2d+w U6=du+2d+2w
// V-slot offsets: V1=dv V2=dv+d V3=dv+d+h

interface FaceDef {
	n: [number, number, number]
	// 4 vertex indices (in Corner order above) + 4 UV pixels (u,v) per Polygon remap
	idx: [number, number, number, number]
	uv: [number, number][]
}

function buildCubeGeo(cube: CubeDef): THREE.BufferGeometry {
	const { du, dv, x, y, z, w, h, d, inflate } = cube
	const texW = TEX_SIZE
	const texH = TEX_SIZE
	const ex = inflate
	const minX = x - ex
	const maxX = x + w + ex
	const minY = y - ex
	const maxY = y + h + ex
	const minZ = z - ex
	const maxZ = z + d + ex
	const V = [
		[minX, minY, minZ],
		[maxX, minY, minZ],
		[maxX, maxY, minZ],
		[minX, maxY, minZ],
		[minX, minY, maxZ],
		[maxX, minY, maxZ],
		[maxX, maxY, maxZ],
		[minX, maxY, maxZ],
	]

	const U1 = du
	const U2 = du + d
	const U3 = du + d + w
	const U4 = du + d + 2 * w
	const U5 = du + 2 * d + w
	const U6 = du + 2 * d + 2 * w
	const V1 = dv
	const V2 = dv + d
	const V3 = dv + d + h

	const faces: FaceDef[] = [
		{ n: [0, -1, 0], idx: [5, 4, 0, 1], uv: [[U3, V1], [U2, V1], [U2, V2], [U3, V2]] }, // DOWN(-y)
		{ n: [0, 1, 0], idx: [2, 3, 7, 6], uv: [[U4, V2], [U3, V2], [U3, V1], [U4, V1]] }, // UP(+y)
		{ n: [-1, 0, 0], idx: [0, 4, 7, 3], uv: [[U2, V2], [U1, V2], [U1, V3], [U2, V3]] }, // WEST(-x)
		{ n: [0, 0, -1], idx: [1, 0, 3, 2], uv: [[U3, V2], [U2, V2], [U2, V3], [U3, V3]] }, // NORTH(-z)
		{ n: [1, 0, 0], idx: [5, 1, 2, 6], uv: [[U5, V2], [U3, V2], [U3, V3], [U5, V3]] }, // EAST(+x)
		{ n: [0, 0, 1], idx: [4, 5, 6, 7], uv: [[U6, V2], [U5, V2], [U5, V3], [U6, V3]] }, // SOUTH(+z)
	]

	// Each cube corner can carry a different UV per face, so we can't share verts:
	// emit 6 faces × 4 corners, indexed via BufferGeometry.addGroup-free index buffer.
	const positions: number[] = []
	const normals: number[] = []
	const uvs: number[] = []
	for (const face of faces) {
		const base = positions.length / 3
		for (let k = 0; k < 4; k++) {
			const vi = face.idx[k]
			positions.push(V[vi][0], V[vi][1], V[vi][2])
			normals.push(face.n[0], face.n[1], face.n[2])
			const [u, v] = face.uv[k]
			uvs.push(u / texW, 1 - v / texH)
		}
	}

	const indices = []
	for (let f = 0; f < faces.length; f++) {
		const base = f * 4
		indices.push(base, base + 1, base + 2, base, base + 2, base + 3)
	}

	const geo = new THREE.BufferGeometry()
	geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3))
	geo.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3))
	geo.setAttribute('uv', new THREE.Float32BufferAttribute(uvs, 2))
	geo.setIndex(indices)
	return geo
}

function buildBoxMesh(tex: THREE.Texture, cube: CubeDef): THREE.Mesh {
	// Cutout only (like MC's entityCutoutNoCull): no transparency blending, so layered
	// faces don't visually stack and the render order stays stable.
	const mat = new THREE.MeshLambertMaterial({
		map: tex,
		alphaTest: 0.05,
		side: THREE.DoubleSide,
	})
	return new THREE.Mesh(buildCubeGeo(cube), mat)
}

function main() {
	const canvas = document.querySelector<HTMLCanvasElement>('#about_scene')
	if (!canvas) return console.error('No canvas')

	let width = 0
	let height = 0
	let dpr = 1

	function resize() {
		const rect = canvas.getBoundingClientRect()
		width = rect.width
		height = rect.height
		dpr = window.devicePixelRatio || 1
	}

	resize()

	const renderer = new THREE.WebGLRenderer({
		canvas,
		alpha: true,
		antialias: false,
	})
	renderer.setClearColor(0x000000, 0)

	const scene = new THREE.Scene()
	// Wide depth range: the model is scaled up to ~19×, so its world z spans well past
	// the default near/far planes and would get clipped (head front, wing tips).
	const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, -1000, 1000)
	camera.position.set(0, 0, 30)
	camera.lookAt(0, 0, 0)

	scene.add(new THREE.AmbientLight(0xffffff, 1.7))
	const sun = new THREE.DirectionalLight(0xffffff, 1.3)
	sun.position.set(2, 5, 8)
	scene.add(sun)
	const rim = new THREE.DirectionalLight(0x88ccff, 0.7)
	rim.position.set(-3, -2, -4)
	scene.add(rim)

	let model: THREE.Group | null = null
	let poseGroup: THREE.Group | null = null
	let headBone: THREE.Group | null = null
	let bodyBone: THREE.Group | null = null
	let wingRight: THREE.Group | null = null
	let wingLeft: THREE.Group | null = null
	let armRight: THREE.Group | null = null
	let armLeft: THREE.Group | null = null
	let itemMesh: THREE.Mesh | null = null

	function buildModel(tex: THREE.Texture) {
		model = new THREE.Group()
		// Entity models use +Y = down in Minecraft; flip so head points up on screen.
		const flip = new THREE.Group()
		flip.scale.set(1, -1, 1)
		model.add(flip)
		const pose = new THREE.Group()
		// The allay faces -z (NORTH) in model space; turn it to face the camera at +z, then add the pose yaw.
		pose.rotation.y = Math.PI + POSE_YAW
		flip.add(pose)
		poseGroup = pose

		const boneGroups = new Map<string, THREE.Group>()
		for (const part of PARTS) {
			const [bx, by, bz] = part.offset
			const bone = new THREE.Group()
			bone.position.set(bx, by, bz)
			for (const cube of part.cubes) bone.add(buildBoxMesh(tex, cube))
			boneGroups.set(part.name, bone)
			const parentName = part.parent === 'root' ? null : part.parent
			const parent = parentName ? boneGroups.get(parentName) : pose
			parent?.add(bone)
			if (part.name === 'head') headBone = bone
			else if (part.name === 'body') bodyBone = bone
			else if (part.name === 'right_wing') wingRight = bone
			else if (part.name === 'left_wing') wingLeft = bone
			else if (part.name === 'right_arm') armRight = bone
			else if (part.name === 'left_arm') armLeft = bone
		}
		scene.add(model)
	}

	function buildItem() {
		const tex = new THREE.TextureLoader().load(diamondTextureUrl, (texture) => {
			texture.magFilter = THREE.NearestFilter
			texture.minFilter = THREE.NearestFilter
			const geo = new THREE.BoxGeometry(1.1, 1.1, 1.1)
			const mat = new THREE.MeshLambertMaterial({
				map: texture,
				alphaTest: 0.05,
			})
			itemMesh = new THREE.Mesh(geo, mat)
			// MC coords inside the pose group (front is -z, +y is down):
			// in front of the raised hands.
			itemMesh.position.set(0, -1.7, -2.9)
			itemMesh.rotation.set(0, Math.PI / 4, 0)
			poseGroup?.add(itemMesh)
		})
		void tex
	}

	function updateCamera() {
		const halfW = width / 2
		const halfH = height / 2
		camera.left = -halfW
		camera.right = halfW
		camera.top = halfH
		camera.bottom = -halfH
		camera.updateProjectionMatrix()
	}

	function resizeCanvas() {
		resize()
		renderer.setPixelRatio(dpr)
		renderer.setSize(width, height, false)
		updateCamera()
	}

	// ── Allay state ────────────────────────────────────────────────
	let anchorX = 0
	let anchorY = 0
	let moveFromX = 0
	let moveFromY = 0
	let moveToX = 0
	let moveToY = 0
	let sizeFrom = 1
	let sizeTo = 1
	let currentSize = 1

	let phase: 'hover' | 'move' = 'hover'
	let phaseElapsed = 0
	let hoverDuration = HOVER_MIN_MS

	let mouseSpeed = 0
	let lastMouseX = 0
	let lastMouseY = 0
	let lastMouseTime = performance.now()
	let mouseRelX = 0
	let mouseRelY = 0
	let isPointerDown = false

	let flapPhase = 0
	let flapSpeed = 3.2
	// Vanilla setupAnim's flyF = min(walkAnimationSpeed/0.3, 1): 0 while hovering, 1 while moving.
	let moveBlend = 0

	function baseSize() {
		return Math.min(height * 0.62, 170)
	}

	function clampPosition(value: number, half: number, limit: number) {
		const min = half + EDGE_MARGIN
		const max = limit - half - EDGE_MARGIN
		if (max < min) return limit / 2
		return Math.min(Math.max(value, min), max)
	}

	function pickHoverTarget() {
		const half = (baseSize() * sizeTo) / 2
		moveToX = clampPosition(
			EDGE_MARGIN + Math.random() * (width - EDGE_MARGIN * 2),
			half,
			width,
		)
		moveToY = clampPosition(
			EDGE_MARGIN + Math.random() * (height - EDGE_MARGIN * 2),
			half,
			height,
		)
		sizeTo = SIZE_MIN + Math.random() * (SIZE_MAX - SIZE_MIN)
	}

	function update(deltaMs: number) {
		phaseElapsed += deltaMs

		if (phase === 'hover') {
			anchorX = moveToX
			anchorY = moveToY
			currentSize += (sizeTo - currentSize) * Math.min(deltaMs / 400, 1)

			if (phaseElapsed >= hoverDuration) {
				phase = 'move'
				phaseElapsed = 0
				moveFromX = anchorX
				moveFromY = anchorY
				sizeFrom = currentSize
				pickHoverTarget()
			}
		} else {
			const progress = Math.min(phaseElapsed / MOVE_DURATION_MS, 1)
			const eased =
				progress < 0.5 ? 2 * progress * progress : 1 - Math.pow(-2 * progress + 2, 2) / 2
			anchorX = moveFromX + (moveToX - moveFromX) * eased
			anchorY = moveFromY + (moveToY - moveFromY) * eased
			currentSize = sizeFrom + (sizeTo - sizeFrom) * eased

			if (progress >= 1) {
				phase = 'hover'
				phaseElapsed = 0
				hoverDuration = HOVER_MIN_MS + Math.random() * (HOVER_MAX_MS - HOVER_MIN_MS)
			}
		}

		// Vanilla flaps at a constant ~6.98 rad/s (ageInTicks*20*π/180 per tick ×20 tps);
		// keep the interactive speed boost on top of that.
		const baseFlap = 6.98
		const targetSpeed = baseFlap * (1 + mouseSpeed * MOUSE_SPEED_BOOST + (isPointerDown ? 2.2 : 0))
		flapSpeed += (targetSpeed - flapSpeed) * Math.min(deltaMs / 500, 1)
		flapPhase += (deltaMs / 1000) * flapSpeed

		const moveTarget = phase === 'move' ? 1 : 0
		moveBlend += (moveTarget - moveBlend) * Math.min(deltaMs / 500, 1)
	}

	function draw(elapsedMs: number) {
		if (!model) return
		const size = baseSize() * currentSize
		const unitsPerPx = size / MODEL_HEIGHT

		const bobScale = phase === 'hover' ? 1 : 0.35
		const bob = Math.sin(elapsedMs * 0.001 * BOB_FREQUENCY) * BOB_AMPLITUDE * bobScale

		const wx = anchorX - width / 2
		const wy = height / 2 - (anchorY + bob) - 4.5 * unitsPerPx
		model.position.set(wx, wy, 0)
		model.scale.setScalar(unitsPerPx)

		const dvx = phase === 'move' ? moveToX - moveFromX : 0
		const dvy = phase === 'move' ? moveToY - moveFromY : 0
		const dirSpeed = Math.hypot(dvx, dvy)

		if (headBone) {
			const headTurn =
				-POSE_YAW + THREE.MathUtils.clamp(-dvx * 0.0008, -0.9, 0.9) * Math.min(dirSpeed * 0.006, 1)
			headBone.rotation.y =
				headTurn + THREE.MathUtils.clamp(mouseRelX * 0.003, -0.6, 0.6) * (isPointerDown ? 0.25 : 1)
			headBone.rotation.x = THREE.MathUtils.clamp(mouseRelY * 0.003, -0.4, 0.4)
		}
		// ── Vanilla AllayModel.setupAnim port (non-dancing branch, holding an item) ──
		// The flip group mirrors the whole MC world (display = S·R_mc·v == S·R_3js·v),
		// so bone rotation angles copy the MC values verbatim — no sign flips.
		const flyF = moveBlend
		// f5 = cos(k)*π*0.15 + walkAnimationSpeed  (k == flapPhase here)
		const flap = Math.cos(flapPhase) * (Math.PI * 0.15) + flyF * 0.9
		if (wingRight && wingLeft) {
			// xRot = 25° when hovering, →0 while moving; yRot = ∓45° ± flap (the actual flapping axis)
			const fold = 0.43633232 * (1 - flyF)
			wingRight.rotation.x = fold
			wingLeft.rotation.x = fold
			wingRight.rotation.y = -0.7853982 + flap
			wingLeft.rotation.y = 0.7853982 - flap
		}
		if (bodyBone) {
			// body.xRot = flyF * 45° (leans forward while moving)
			bodyBone.rotation.x = flyF * 0.7853982
			bodyBone.rotation.z = Math.sin(elapsedMs * 0.0012) * 0.05 + dvx * 0.00012
		}
		if (armRight && armLeft) {
			// hold=1: armSwing = lerp(flyF, -1.0472, -1.1345) → arms raised holding the item;
			// zRot = ±25° spread, yRot = ∓16° inward (all holdingAnimationProgress terms).
			const raise = -1.0471976 - 0.0872664 * flyF
			armRight.rotation.x = raise
			armLeft.rotation.x = raise
			armRight.rotation.z = 0.43633232
			armLeft.rotation.z = -0.43633232
			armRight.rotation.y = 0.27925268
			armLeft.rotation.y = -0.27925268
		}
		if (itemMesh) itemMesh.rotation.y += 0.02
	}

	function onMouseMove(event: MouseEvent) {
		const now = performance.now()
		const dt = Math.max(now - lastMouseTime, 1)
		const dx = event.clientX - lastMouseX
		const dy = event.clientY - lastMouseY
		const instant = Math.hypot(dx, dy) / dt
		mouseSpeed += (instant - mouseSpeed) * MOUSE_SPEED_SMOOTH

		const rect = canvas.getBoundingClientRect()
		mouseRelX = event.clientX - (rect.left + rect.width / 2)
		mouseRelY = event.clientY - (rect.top + rect.height / 2)

		lastMouseX = event.clientX
		lastMouseY = event.clientY
		lastMouseTime = now
	}

	let isUpdating = true
	let lastTime = performance.now()
	const startTime = lastTime

	function animate(now: number) {
		if (!isUpdating) return
		requestAnimationFrame(animate)

		const deltaMs = Math.min(now - lastTime, 64)
		lastTime = now
		const elapsedMs = now - startTime

		if (!model) return

		update(deltaMs)
		draw(elapsedMs)
		renderer.render(scene, camera)
	}

	function positionInitially() {
		const half = (baseSize() * currentSize) / 2
		moveToX = clampPosition(width / 2, half, width)
		moveToY = clampPosition(height / 2, half, height)
		anchorX = moveToX
		anchorY = moveToY
		hoverDuration = HOVER_MIN_MS + Math.random() * (HOVER_MAX_MS - HOVER_MIN_MS)
	}

	new THREE.TextureLoader().load(allayTextureUrl, (texture) => {
		texture.magFilter = THREE.NearestFilter
		texture.minFilter = THREE.NearestFilter
		buildModel(texture)
		buildItem()
		positionInitially()
		updateCamera()
	})

	function onPointerDown() {
		isPointerDown = true
	}

	function onPointerUp() {
		isPointerDown = false
	}

	const ro2 = new ResizeObserver(resizeCanvas)
	ro2.observe(canvas)
	resizeCanvas()

	addEventListener('mousemove', onMouseMove)
	canvas.addEventListener('pointerdown', onPointerDown)
	window.addEventListener('pointerup', onPointerUp)
	requestAnimationFrame(animate)

	onScopeDispose(() => {
		isUpdating = false
		removeEventListener('mousemove', onMouseMove)
		canvas.removeEventListener('pointerdown', onPointerDown)
		window.removeEventListener('pointerup', onPointerUp)
		ro2.disconnect()
	})
}

onMounted(main)
</script>

<style>
#about_scene {
	background: linear-gradient(
		to bottom,
		color-mix(in srgb, var(--color-brand) 36%, var(--surface-1) 100%),
		#00000000 40%
	);
}
</style>