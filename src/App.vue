<script setup>
import { computed, onBeforeUnmount, onMounted, ref, shallowRef, watch } from 'vue'
import * as THREE from 'three'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import { MTLLoader } from 'three/examples/jsm/loaders/MTLLoader.js'
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader.js'

const viewRef = ref(null)
const status = ref('正在读取 demo.obj …')
const error = ref('')
const query = ref('')
const wireframe = ref(false)
const hideWall = ref(true)
const ambientOn = ref(true)
const ambientIntensity = ref(1.3)
const hemisphereIntensity = ref(0.75)
const showTransparency = ref(0.5)
const zoom = ref(1)
const panX = ref(0)
const panY = ref(0)
const panelOpen = ref(false)
const parts = ref([])
const selectedName = ref('')

const model = shallowRef(null)
let renderer
let scene
let camera
let controls
let helper
let ambientLight
let hemisphereLight
let frameId = 0
let homeDistance = 0
const baseTarget = new THREE.Vector3()
const viewSize = new THREE.Vector3()

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return parts.value
  return parts.value.filter((item) => item.name.toLowerCase().includes(q) || item.kind.includes(q))
})

function kindOf(name) {
  if (name.endsWith('_drawer')) return '抽屉'
  if (name.endsWith('_show')) return '展示'
  if (name === 'wall') return '墙'
  if (name === 'floor') return '地面'
  return '柜体'
}

function rewriteMtl(text) {
  return text.replace(/^(\s*map_\w+\s+)(.+)$/gm, (_, key, rawPath) => {
    const file = rawPath.trim().split(/[/\\]/).pop()
    return `${key}${file}`
  })
}

function frameFront() {
  if (!model.value || !camera || !controls) return
  const box = new THREE.Box3()
  model.value.children.forEach((child) => {
    if (child.name === 'wall' || child.name === 'floor') return
    box.expandByObject(child)
  })
  if (box.isEmpty()) return
  const size = box.getSize(new THREE.Vector3())
  const center = box.getCenter(new THREE.Vector3())
  const aspect = camera.aspect || 1
  const halfFov = (camera.fov * Math.PI) / 360
  const dist = (Math.max(size.y, size.x / aspect) / 2 / Math.tan(halfFov)) * 1.2
  camera.near = Math.max(dist / 1000, 0.01)
  camera.far = dist * 20 + size.z
  camera.updateProjectionMatrix()
  camera.position.set(center.x, center.y, center.z + dist)
  controls.target.copy(center)
  controls.update()
  baseTarget.copy(center)
  viewSize.copy(size)
  homeDistance = dist
  if (panX.value !== 0) panX.value = 0
  if (panY.value !== 0) panY.value = 0
  if (zoom.value !== 1) zoom.value = 1
}

function applyWireframe(enabled) {
  model.value?.traverse((child) => {
    if (!child.isMesh) return
    const mats = Array.isArray(child.material) ? child.material : [child.material]
    mats.forEach((mat) => {
      mat.wireframe = enabled
    })
  })
}

function toggleWireframe() {
  wireframe.value = !wireframe.value
  applyWireframe(wireframe.value)
}

function applyWallVisibility() {
  const wall = parts.value.find((item) => item.name === 'wall')
  if (!wall) return
  wall.object.visible = !hideWall.value
  if (selectedName.value === 'wall' && helper) helper.visible = wall.object.visible
}

watch(hideWall, applyWallVisibility)

function applyAmbient() {
  if (ambientLight) ambientLight.visible = ambientOn.value
}

watch(ambientOn, applyAmbient)

function applyLightIntensities() {
  if (ambientLight) ambientLight.intensity = Number(ambientIntensity.value)
  if (hemisphereLight) hemisphereLight.intensity = Number(hemisphereIntensity.value)
}

watch([ambientIntensity, hemisphereIntensity], applyLightIntensities)

function applyShowTransparency() {
  if (!model.value) return
  const opacity = 1 - Number(showTransparency.value)
  const seen = new Set()
  model.value.traverse((child) => {
    if (!child.isMesh) return
    const mats = Array.isArray(child.material) ? child.material : [child.material]
    mats.forEach((mat) => {
      if (!mat || mat.name !== 'show' || seen.has(mat)) return
      seen.add(mat)
      mat.opacity = opacity
      mat.transparent = opacity < 1
    })
  })
  parts.value.forEach((part) => {
    if (part.name.endsWith('_show')) part.object.visible = opacity > 0
  })
}

watch(showTransparency, applyShowTransparency)

function applyView() {
  if (!camera || !controls || !homeDistance) return
  const nextZoom = Number(zoom.value)
  if (!Number.isFinite(nextZoom) || nextZoom <= 0) return
  const look = new THREE.Vector3()
  camera.getWorldDirection(look)
  if (look.lengthSq() < 1e-8) look.set(0, 0, -1)
  const right = new THREE.Vector3().crossVectors(look, camera.up).normalize()
  const up = new THREE.Vector3().crossVectors(right, look).normalize()
  const target = baseTarget.clone()
  target.addScaledVector(right, Number(panX.value) * viewSize.x * 0.5)
  target.addScaledVector(up, Number(panY.value) * viewSize.y * 0.5)
  const dist = homeDistance / nextZoom
  camera.position.copy(target).addScaledVector(look, -dist)
  controls.target.copy(target)
  camera.near = Math.max(dist / 1000, 0.01)
  camera.far = dist * 50
  camera.updateProjectionMatrix()
  controls.update()
}

watch([zoom, panX, panY], applyView)

let pointerDown = null

function onPointerDown(event) {
  pointerDown = { x: event.clientX, y: event.clientY }
}

function selectPart(name) {
  const part = parts.value.find((item) => item.name === name)
  if (!part) return
  selectedName.value = name
  helper.setFromObject(part.object)
  helper.visible = true
}

function onPointerUp(event) {
  if (!pointerDown || !model.value || !renderer) return
  const dx = event.clientX - pointerDown.x
  const dy = event.clientY - pointerDown.y
  pointerDown = null
  if (dx * dx + dy * dy > 9) return

  const rect = renderer.domElement.getBoundingClientRect()
  const pointer = new THREE.Vector2(
    ((event.clientX - rect.left) / rect.width) * 2 - 1,
    -((event.clientY - rect.top) / rect.height) * 2 + 1,
  )
  const raycaster = new THREE.Raycaster()
  raycaster.setFromCamera(pointer, camera)
  const hits = raycaster.intersectObject(model.value, true)
  const hit = hits.find((item) => {
    let node = item.object
    while (node) {
      if (!node.visible) return false
      node = node.parent
    }
    return true
  })
  if (!hit) return
  let node = hit.object
  while (node.parent && node.parent !== model.value) node = node.parent
  if (node.name) selectPart(node.name)
}

function onResize() {
  if (!renderer || !camera || !viewRef.value) return
  const { clientWidth, clientHeight } = viewRef.value
  camera.aspect = clientWidth / Math.max(clientHeight, 1)
  camera.updateProjectionMatrix()
  renderer.setSize(clientWidth, clientHeight)
}

function togglePanel() {
  panelOpen.value = !panelOpen.value
}

async function loadModel() {
  const mtlText = await fetch('/demo.mtl').then((res) => {
    if (!res.ok) throw new Error(`读取 demo.mtl 失败 (${res.status})`)
    return res.text()
  })

  const mtlLoader = new MTLLoader()
  mtlLoader.setMaterialOptions({ side: THREE.DoubleSide })
  const materials = mtlLoader.parse(rewriteMtl(mtlText), '/')
  materials.preload()

  const object = await new OBJLoader().setMaterials(materials).loadAsync('/demo.obj')
  object.traverse((child) => {
    if (child.isMesh) child.castShadow = false
  })

  scene.add(object)
  model.value = object
  helper = new THREE.BoxHelper(object, 0xf0a202)
  helper.visible = false
  scene.add(helper)

  parts.value = object.children
    .filter((child) => child.name)
    .map((child) => ({
      name: child.name,
      kind: kindOf(child.name),
      object: child,
    }))

  applyWallVisibility()
  frameFront()
  status.value = `${parts.value.length} 个物体`
}

function animate() {
  frameId = requestAnimationFrame(animate)
  controls?.update()
  renderer?.render(scene, camera)
}

onMounted(async () => {
  const host = viewRef.value
  scene = new THREE.Scene()
  scene.background = new THREE.Color('#d7dde6')

  camera = new THREE.PerspectiveCamera(45, 1, 0.1, 5000)
  renderer = new THREE.WebGLRenderer({ antialias: true })
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
  renderer.outputColorSpace = THREE.SRGBColorSpace
  host.appendChild(renderer.domElement)

  controls = new OrbitControls(camera, renderer.domElement)
  controls.enableDamping = true

  ambientLight = new THREE.AmbientLight(0xffffff, ambientIntensity.value)
  ambientLight.visible = ambientOn.value
  scene.add(ambientLight)
  const sun = new THREE.DirectionalLight(0xffffff, 1.1)
  sun.position.set(400, 700, 300)
  scene.add(sun)
  hemisphereLight = new THREE.HemisphereLight(0xf4f7fb, 0x8d97a3, hemisphereIntensity.value)
  scene.add(hemisphereLight)

  window.addEventListener('resize', onResize)
  renderer.domElement.addEventListener('pointerdown', onPointerDown)
  renderer.domElement.addEventListener('pointerup', onPointerUp)
  onResize()
  animate()

  try {
    await loadModel()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '模型加载失败'
    status.value = '加载失败'
  }
})

onBeforeUnmount(() => {
  cancelAnimationFrame(frameId)
  window.removeEventListener('resize', onResize)
  controls?.dispose()
  renderer?.dispose()
  renderer?.domElement.remove()
})
</script>

<template>
  <div class="app">
    <button
      type="button"
      class="panel-toggle"
      :aria-expanded="panelOpen"
      :aria-label="panelOpen ? '隐藏模型查看面板' : '打开模型查看面板'"
      @click="togglePanel"
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M4 6h16M4 12h16M4 18h10" />
      </svg>
    </button>
    <aside v-show="panelOpen">
      <header>
        <h1>模型查看</h1>
        <p>{{ status }}</p>
      </header>
      <p v-if="error" class="error">{{ error }}</p>
      <label class="search">
        <input v-model="query" type="search" placeholder="搜索物体" />
      </label>
      <div class="tools">
        <button type="button" :disabled="!model" @click="frameFront">重置视角</button>
        <button
          type="button"
          :class="{ on: wireframe }"
          :disabled="!model"
          @click="toggleWireframe"
        >
          线框
        </button>
        <label class="check">
          <input v-model="hideWall" type="checkbox" :disabled="!model" />
          隐藏墙
        </label>
        <label class="check">
          <input v-model="ambientOn" type="checkbox" :disabled="!model" />
          开环境光
        </label>
        <label class="slider">
          <span>环境光 {{ Number(ambientIntensity).toFixed(2) }}</span>
          <input v-model.number="ambientIntensity" type="range" min="0" max="3" step="0.01" :disabled="!model" />
        </label>
        <label class="slider">
          <span>半球光 {{ Number(hemisphereIntensity).toFixed(2) }}</span>
          <input
            v-model.number="hemisphereIntensity"
            type="range"
            min="0"
            max="3"
            step="0.01"
            :disabled="!model"
          />
        </label>
        <label class="slider">
          <span>缩放 {{ Math.round(zoom * 100) }}%</span>
          <input v-model.number="zoom" type="range" min="0.4" max="3" step="0.01" :disabled="!model" />
        </label>
        <label class="slider">
          <span>左右 {{ Math.round(panX * 100) }}</span>
          <input v-model.number="panX" type="range" min="-1" max="1" step="0.01" :disabled="!model" />
        </label>
        <label class="slider">
          <span>上下 {{ Math.round(panY * 100) }}</span>
          <input v-model.number="panY" type="range" min="-1" max="1" step="0.01" :disabled="!model" />
        </label>
        <label class="slider">
          <span>蓝色块透明度 {{ Math.round(showTransparency * 100) }}%</span>
          <input
            v-model.number="showTransparency"
            type="range"
            min="0"
            max="1"
            step="0.01"
            :disabled="!model"
          />
        </label>
      </div>
      <ul>
        <li v-for="item in filtered" :key="item.name">
          <button
            type="button"
            :class="{ on: selectedName === item.name }"
            @click="selectPart(item.name)"
          >
            <span>{{ item.name }}</span>
            <em>{{ item.kind }}</em>
          </button>
        </li>
      </ul>
    </aside>
    <main ref="viewRef"></main>
  </div>
</template>

<style>
* { box-sizing: border-box; }
html, body, #app { margin: 0; height: 100%; }
body { font-family: "Segoe UI", "PingFang SC", sans-serif; color: #1c2430; }
.app { position: relative; display: flex; height: 100%; }
.panel-toggle {
  position: absolute;
  top: 12px;
  left: 12px;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  padding: 0;
  border: 1px solid #cfd6e0;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 4px rgba(28, 36, 48, 0.16);
  cursor: pointer;
}
.panel-toggle svg {
  width: 18px;
  height: 18px;
  stroke: #1c2430;
  stroke-width: 2;
  stroke-linecap: round;
  fill: none;
}
.panel-toggle:hover { background: #f3f6fa; }
aside {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  z-index: 1;
  width: 300px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  overflow: hidden;
  background: #f7f8fa;
  border-right: 1px solid #d9dee7;
  box-shadow: 2px 0 12px rgba(28, 36, 48, 0.12);
}
header { padding-left: 40px; }
header h1 { margin: 0; font-size: 18px; }
header p { margin: 4px 0 0; color: #5d6b7c; font-size: 13px; }
.error { margin: 0; color: #a32020; font-size: 13px; }
.search input, .tools button, li button {
  width: 100%;
  border: 1px solid #cfd6e0;
  background: #fff;
  border-radius: 8px;
  font: inherit;
}
.search input { padding: 8px 10px; }
.tools { display: flex; flex-wrap: wrap; gap: 8px; }
.tools button, li button { padding: 8px 10px; cursor: pointer; }
.tools button { flex: 1; }
.check {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  font-size: 14px;
  cursor: pointer;
}
.check input:disabled { cursor: default; }
.slider {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 100%;
  font-size: 14px;
}
.slider input { width: 100%; margin: 0; }
.slider input:disabled { cursor: default; }
.tools button.on, li button.on { border-color: #e08900; background: #fff6e8; }
button:disabled { cursor: default; opacity: 0.5; }
ul { flex: 1; min-height: 0; list-style: none; margin: 0; padding: 0; overflow: auto; }
li + li { margin-top: 6px; }
li button { display: flex; justify-content: space-between; gap: 8px; text-align: left; }
li span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
li em { flex: none; color: #6b7787; font-style: normal; font-size: 12px; }
main { flex: 1; min-width: 0; }
main canvas { display: block; width: 100%; height: 100%; }
</style>
