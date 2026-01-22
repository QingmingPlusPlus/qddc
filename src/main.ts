import './style.css'
import { Engine, Sprite, SamplingMethod, PerformanceMetrics } from './engine'

// 状态
let engine: Engine | null = null
let spriteCount = 0
const sprites: Map<number, Sprite> = new Map()

// FPS 计算
let lastFrameTime = performance.now()
let frameCount = 0
let fps = 0

// 性能监控
let perfEnabled = true
let lastPerfMetrics: PerformanceMetrics | null = null
let perfUpdateCounter = 0
const PERF_UPDATE_INTERVAL = 10  // 每 10 帧更新一次性能显示

// 动画状态
let isAnimating = false
let animationLastTime = 0
const ROTATION_SPEED = (2 * Math.PI) / 6000  // 6秒转一圈（弧度/毫秒）

// 颜色列表，用于创建不同颜色的精灵图
const COLORS = [
  [255, 0, 0],     // 红
  [0, 255, 0],     // 绿
  [0, 0, 255],     // 蓝
  [255, 255, 0],   // 黄
  [255, 0, 255],   // 品红
  [0, 255, 255],   // 青
  [255, 128, 0],   // 橙
  [128, 0, 255],   // 紫
]

// 平移和旋转步进
const TRANSLATE_STEP = 20
const ROTATE_STEP = Math.PI / 12  // 15 度
const SCALE_STEP = 1.2
const ZINDEX_STEP = 1

/**
 * 加载图片文件并转换为 RGBA 像素数据
 * @param file 图片文件 (jpg/png)
 * @returns Promise<{data: Uint8Array, width: number, height: number}>
 */
async function loadImageAsRGBA(file: File): Promise<{ data: Uint8Array; width: number; height: number }> {
  return new Promise((resolve, reject) => {
    // 验证文件类型
    if (!file.type.match(/^image\/(jpeg|png)$/)) {
      reject(new Error('仅支持 JPG 和 PNG 格式的图片'))
      return
    }

    const img = new Image()
    const url = URL.createObjectURL(file)

    img.onload = () => {
      // 创建离屏 canvas 来获取像素数据
      const canvas = document.createElement('canvas')
      canvas.width = img.width
      canvas.height = img.height

      const ctx = canvas.getContext('2d')
      if (!ctx) {
        URL.revokeObjectURL(url)
        reject(new Error('无法创建 Canvas 上下文'))
        return
      }

      // 绘制图片到 canvas
      ctx.drawImage(img, 0, 0)

      // 获取 RGBA 像素数据
      const imageData = ctx.getImageData(0, 0, img.width, img.height)

      // 转换为 Uint8Array (ImageData.data 是 Uint8ClampedArray)
      const rgbaData = new Uint8Array(imageData.data.buffer)

      // 清理
      URL.revokeObjectURL(url)

      resolve({
        data: rgbaData,
        width: img.width,
        height: img.height
      })
    }

    img.onerror = () => {
      URL.revokeObjectURL(url)
      reject(new Error('图片加载失败'))
    }

    img.src = url
  })
}

/**
 * 创建精灵图控制卡片
 */
function createSpriteCard(sprite: Sprite, color: number[]): HTMLElement {
  const card = document.createElement('div')
  card.className = 'sprite-card'
  card.id = `sprite-card-${sprite.id}`

  // 颜色预览
  const colorPreview = `rgb(${color[0]}, ${color[1]}, ${color[2]})`

  card.innerHTML = `
    <div class="sprite-card-header">
      <div class="sprite-color" style="background: ${colorPreview}"></div>
      <span class="sprite-name">精灵图 #${sprite.id}</span>
      <button class="btn-icon btn-delete" data-sprite-id="${sprite.id}" title="删除">×</button>
    </div>
    <div class="sprite-card-info">
      <span class="info-label">位置:</span> <span class="pos-x">${sprite.position.x.toFixed(1)}</span>, <span class="pos-y">${sprite.position.y.toFixed(1)}</span><br>
      <span class="info-label">旋转:</span> <span class="rotation">${(sprite.rotation * 180 / Math.PI).toFixed(1)}</span>°<br>
      <span class="info-label">缩放:</span> <span class="scale-x">${sprite.scale.x.toFixed(2)}</span>, <span class="scale-y">${sprite.scale.y.toFixed(2)}</span><br>
      <span class="info-label">层级:</span> <span class="zindex">${sprite.zindex}</span>
    </div>
    <div class="sprite-card-controls">
      <div class="control-row">
        <span class="control-label">平移</span>
        <div class="control-buttons">
          <button class="btn-mini" data-action="translate-left" data-sprite-id="${sprite.id}">←</button>
          <button class="btn-mini" data-action="translate-up" data-sprite-id="${sprite.id}">↑</button>
          <button class="btn-mini" data-action="translate-down" data-sprite-id="${sprite.id}">↓</button>
          <button class="btn-mini" data-action="translate-right" data-sprite-id="${sprite.id}">→</button>
        </div>
      </div>
      <div class="control-row">
        <span class="control-label">旋转</span>
        <div class="control-buttons">
          <button class="btn-mini" data-action="rotate-ccw" data-sprite-id="${sprite.id}">↺</button>
          <button class="btn-mini" data-action="rotate-cw" data-sprite-id="${sprite.id}">↻</button>
        </div>
      </div>
      <div class="control-row">
        <span class="control-label">缩放</span>
        <div class="control-buttons">
          <button class="btn-mini" data-action="scale-up" data-sprite-id="${sprite.id}">+</button>
          <button class="btn-mini" data-action="scale-down" data-sprite-id="${sprite.id}">−</button>
        </div>
      </div>
      <div class="control-row">
        <span class="control-label">层级</span>
        <div class="control-buttons">
          <button class="btn-mini" data-action="zindex-up" data-sprite-id="${sprite.id}">▲</button>
          <button class="btn-mini" data-action="zindex-down" data-sprite-id="${sprite.id}">▼</button>
        </div>
      </div>
    </div>
  `

  return card
}

/**
 * 更新精灵图卡片状态显示
 */
function updateSpriteCard(sprite: Sprite) {
  const card = document.getElementById(`sprite-card-${sprite.id}`)
  if (!card) return

  const posX = card.querySelector('.pos-x')
  const posY = card.querySelector('.pos-y')
  const rotation = card.querySelector('.rotation')
  const scaleX = card.querySelector('.scale-x')
  const scaleY = card.querySelector('.scale-y')
  const zindex = card.querySelector('.zindex')

  if (posX) posX.textContent = sprite.position.x.toFixed(1)
  if (posY) posY.textContent = sprite.position.y.toFixed(1)
  if (rotation) rotation.textContent = (sprite.rotation * 180 / Math.PI).toFixed(1)
  if (scaleX) scaleX.textContent = sprite.scale.x.toFixed(2)
  if (scaleY) scaleY.textContent = sprite.scale.y.toFixed(2)
  if (zindex) zindex.textContent = sprite.zindex.toString()
}

/**
 * 更新空状态提示
 */
function updateEmptyHint() {
  const spriteList = document.getElementById('spriteList')!
  const existingHint = spriteList.querySelector('.empty-hint')

  if (sprites.size === 0) {
    if (!existingHint) {
      const hint = document.createElement('p')
      hint.className = 'empty-hint'
      hint.textContent = '暂无精灵图，点击上方按钮创建'
      spriteList.appendChild(hint)
    }
  } else {
    if (existingHint) {
      existingHint.remove()
    }
  }
}

/**
 * 处理精灵图操作
 */
function handleSpriteAction(action: string, spriteId: number) {
  const sprite = sprites.get(spriteId)
  if (!sprite || !engine) return

  switch (action) {
    case 'translate-left':
      sprite.translate(-TRANSLATE_STEP, 0)
      break
    case 'translate-right':
      sprite.translate(TRANSLATE_STEP, 0)
      break
    case 'translate-up':
      sprite.translate(0, -TRANSLATE_STEP)
      break
    case 'translate-down':
      sprite.translate(0, TRANSLATE_STEP)
      break
    case 'rotate-ccw':
      sprite.rotate(-ROTATE_STEP)
      break
    case 'rotate-cw':
      sprite.rotate(ROTATE_STEP)
      break
    case 'scale-up':
      sprite.scaleBy(SCALE_STEP, SCALE_STEP)
      break
    case 'scale-down':
      sprite.scaleBy(1 / SCALE_STEP, 1 / SCALE_STEP)
      break
    case 'zindex-up':
      engine.setSpriteZIndex(sprite, sprite.zindex + ZINDEX_STEP)
      break
    case 'zindex-down':
      engine.setSpriteZIndex(sprite, sprite.zindex - ZINDEX_STEP)
      break
  }

  updateSpriteCard(sprite)
}

/**
 * 删除精灵图
 */
function removeSprite(spriteId: number) {
  const sprite = sprites.get(spriteId)
  if (!sprite || !engine) return

  engine.removeSprite(sprite)
  sprites.delete(spriteId)

  const card = document.getElementById(`sprite-card-${spriteId}`)
  if (card) {
    card.remove()
  }

  updateEmptyHint()
  console.log(`Removed sprite ${spriteId}`)
}

/**
 * 清空所有精灵图
 */
function clearAllSprites() {
  if (!engine) return

  // 删除所有精灵
  for (const [spriteId, sprite] of sprites) {
    engine.removeSprite(sprite)
    const card = document.getElementById(`sprite-card-${spriteId}`)
    if (card) {
      card.remove()
    }
  }

  sprites.clear()
  updateEmptyHint()
  console.log('Cleared all sprites')
}

/**
 * 初始化应用
 */
async function main() {
  const canvas = document.getElementById('canvas') as HTMLCanvasElement

  if (!canvas) {
    console.error('Canvas not found')
    return
  }

  // 创建引擎
  engine = await Engine.create(canvas)

  // 设置深蓝色背景
  engine.setBackgroundColor(20, 30, 48, 255)

  // FPS 显示元素
  const fpsDisplay = document.getElementById('fpsDisplay')!

  // 性能监控元素
  const perfWasm = document.getElementById('perfWasm')!
  const perfMemory = document.getElementById('perfMemory')!
  const perfCopy = document.getElementById('perfCopy')!
  const perfDraw = document.getElementById('perfDraw')!
  const perfTotal = document.getElementById('perfTotal')!
  const perfSprites = document.getElementById('perfSprites')!
  const perfMaxFps = document.getElementById('perfMaxFps')!
  const barWasm = document.getElementById('barWasm')!
  const barMemory = document.getElementById('barMemory')!
  const barCopy = document.getElementById('barCopy')!
  const barDraw = document.getElementById('barDraw')!
  const perfToggle = document.getElementById('perfToggle')!
  const perfContent = document.getElementById('perfContent')!

  // 性能面板折叠功能
  perfToggle.addEventListener('click', () => {
    perfEnabled = !perfContent.classList.contains('collapsed')
    perfContent.classList.toggle('collapsed')
    perfToggle.textContent = perfContent.classList.contains('collapsed') ? '+' : '−'
  })

  /**
   * 更新性能监控显示
   */
  function updatePerfDisplay(metrics: PerformanceMetrics) {
    // 格式化时间显示
    const formatTime = (ms: number) => ms < 1 ? `${(ms * 1000).toFixed(0)}μs` : `${ms.toFixed(2)}ms`
    
    perfWasm.textContent = formatTime(metrics.wasmRender)
    perfMemory.textContent = formatTime(metrics.memoryRead)
    perfCopy.textContent = formatTime(metrics.imageCopy)
    perfDraw.textContent = formatTime(metrics.canvasDraw)
    perfTotal.textContent = formatTime(metrics.total)
    perfSprites.textContent = metrics.spriteCount.toString()
    
    // 计算理论最大 FPS
    const maxFps = metrics.total > 0 ? Math.round(1000 / metrics.total) : 999
    perfMaxFps.textContent = maxFps > 999 ? '999+' : maxFps.toString()
    
    // 更新进度条 (以总时间为基准计算百分比)
    const total = metrics.total || 1
    barWasm.style.width = `${(metrics.wasmRender / total) * 100}%`
    barMemory.style.width = `${(metrics.memoryRead / total) * 100}%`
    barCopy.style.width = `${(metrics.imageCopy / total) * 100}%`
    barDraw.style.width = `${(metrics.canvasDraw / total) * 100}%`
  }

  // 渲染循环
  function gameLoop() {
    // 计算 FPS
    frameCount++
    const currentTime = performance.now()
    const elapsed = currentTime - lastFrameTime

    if (elapsed >= 1000) {
      fps = Math.round((frameCount * 1000) / elapsed)
      fpsDisplay.textContent = `FPS: ${fps}`
      frameCount = 0
      lastFrameTime = currentTime
    }

    // 动画更新
    if (isAnimating) {
      const deltaTime = animationLastTime > 0 ? currentTime - animationLastTime : 0
      const rotationDelta = ROTATION_SPEED * deltaTime  // 逆时针旋转

      for (const sprite of sprites.values()) {
        sprite.rotate(-rotationDelta)  // 负值表示逆时针
      }

      animationLastTime = currentTime
    }

    if (engine) {
      // 使用带性能计时的渲染
      if (perfEnabled) {
        lastPerfMetrics = engine.renderWithTiming()
        perfUpdateCounter++
        
        // 每隔一定帧数更新性能显示，避免过于频繁
        if (perfUpdateCounter >= PERF_UPDATE_INTERVAL) {
          updatePerfDisplay(lastPerfMetrics)
          perfUpdateCounter = 0
        }
      } else {
        engine.render()
      }
    }
    requestAnimationFrame(gameLoop)
  }
  requestAnimationFrame(gameLoop)

  // 绑定事件
  bindEvents()

  console.log('QDDC Engine initialized!')
}

/**
 * 绑定事件
 */
function bindEvents() {
  const spriteList = document.getElementById('spriteList')!

  // 模式切换按钮
  const modeControlBtn = document.getElementById('modeControl')!
  const modeAnimationBtn = document.getElementById('modeAnimation')!
  const animationControls = document.getElementById('animationControls')!
  const controlElements = document.querySelectorAll('.control-section, .button-group, .batch-create, .sprite-list')

  function setMode(mode: 'control' | 'animation') {
    if (mode === 'control') {
      modeControlBtn.classList.add('active')
      modeAnimationBtn.classList.remove('active')
      controlElements.forEach(el => (el as HTMLElement).style.display = '')
      animationControls.style.display = 'none'
      // 停止动画
      isAnimating = false
      animationLastTime = 0
      const toggleBtn = document.getElementById('toggleAnimation')!
      toggleBtn.textContent = '▶ 开始动画'
      toggleBtn.classList.remove('playing')
    } else {
      modeControlBtn.classList.remove('active')
      modeAnimationBtn.classList.add('active')
      controlElements.forEach(el => (el as HTMLElement).style.display = 'none')
      animationControls.style.display = 'block'
    }
  }

  modeControlBtn.addEventListener('click', () => setMode('control'))
  modeAnimationBtn.addEventListener('click', () => setMode('animation'))

  // 动画开始/停止按钮
  const toggleAnimationBtn = document.getElementById('toggleAnimation')!
  toggleAnimationBtn.addEventListener('click', () => {
    isAnimating = !isAnimating
    if (isAnimating) {
      animationLastTime = performance.now()
      toggleAnimationBtn.textContent = '■ 停止动画'
      toggleAnimationBtn.classList.add('playing')
    } else {
      animationLastTime = 0
      toggleAnimationBtn.textContent = '▶ 开始动画'
      toggleAnimationBtn.classList.remove('playing')
    }
  })

  // 采样方法选择器
  const samplingSelect = document.getElementById('samplingMethod') as HTMLSelectElement
  samplingSelect.addEventListener('change', () => {
    if (!engine) return
    const method = samplingSelect.value as SamplingMethod
    engine.setSamplingMethod(method)
    console.log(`Sampling method changed to: ${method}`)
  })

  // 上传图片按钮
  const uploadBtn = document.getElementById('uploadImage')!
  const imageInput = document.getElementById('imageInput') as HTMLInputElement

  uploadBtn.addEventListener('click', () => {
    imageInput.click()
  })

  imageInput.addEventListener('change', async () => {
    if (!engine || !imageInput.files || imageInput.files.length === 0) return

    const file = imageInput.files[0]

    try {
      // 显示加载状态
      uploadBtn.textContent = '加载中...'
      uploadBtn.setAttribute('disabled', 'true')

      // 加载图片并转换为 RGBA 数据
      const { data, width, height } = await loadImageAsRGBA(file)

      console.log(`Loaded image: ${file.name}, size: ${width}x${height}, bytes: ${data.length}`)

      // 创建精灵图
      const sprite = engine.createSprite(data, width, height)
      spriteCount++

      // 设置位置在场景中心
      sprite.setPosition(0, 0)

      // 设置 z-index
      engine.setSpriteZIndex(sprite, spriteCount)

      // 添加到场景
      engine.addToScene(sprite)

      // 保存到 map
      sprites.set(sprite.id, sprite)

      // 创建控制卡片 (使用白色作为图片精灵的颜色指示)
      const card = createSpriteCard(sprite, [128, 128, 128])
      // 修改卡片标题为文件名
      const nameSpan = card.querySelector('.sprite-name')
      if (nameSpan) {
        nameSpan.textContent = `📷 ${file.name.substring(0, 15)}${file.name.length > 15 ? '...' : ''}`
      }
      spriteList.appendChild(card)

      updateEmptyHint()

      console.log(`Created image sprite ${sprite.id} from ${file.name}`)
    } catch (error) {
      console.error('Failed to load image:', error)
      alert(error instanceof Error ? error.message : '图片加载失败')
    } finally {
      // 恢复按钮状态
      uploadBtn.textContent = '📷 上传图片'
      uploadBtn.removeAttribute('disabled')
      // 清空 input 以便重复选择同一文件
      imageInput.value = ''
    }
  })

  // 创建精灵图
  document.getElementById('createSprite')!.addEventListener('click', () => {
    if (!engine) return

    // 选择颜色
    const color = COLORS[spriteCount % COLORS.length]
    spriteCount++

    // 创建 50x50 的矩形精灵图
    const sprite = engine.createRectSprite(50, 50, color[0], color[1], color[2], 255)

    // 随机位置 (在场景中心附近)
    const offsetX = (Math.random() - 0.5) * 200
    const offsetY = (Math.random() - 0.5) * 200
    sprite.setPosition(offsetX, offsetY)

    // 设置初始 z-index (根据创建顺序)
    engine.setSpriteZIndex(sprite, spriteCount)

    // 添加到场景
    engine.addToScene(sprite)

    // 保存到map
    sprites.set(sprite.id, sprite)

    // 创建控制卡片
    const card = createSpriteCard(sprite, color)
    spriteList.appendChild(card)

    updateEmptyHint()

    console.log(`Created sprite ${sprite.id} at (${offsetX.toFixed(1)}, ${offsetY.toFixed(1)}) with zindex ${sprite.zindex}`)
  })

  // 清空所有精灵图
  document.getElementById('clearAllSprites')!.addEventListener('click', () => {
    if (sprites.size === 0) return
    if (confirm('确定要清空所有精灵图吗？')) {
      clearAllSprites()
    }
  })

  // 批量创建精灵图
  const batchCountInput = document.getElementById('batchCount') as HTMLInputElement
  document.getElementById('batchCreateSprites')!.addEventListener('click', () => {
    if (!engine) return

    const count = Math.min(Math.max(1, parseInt(batchCountInput.value) || 1), 1000)

    for (let i = 0; i < count; i++) {
      // 选择颜色
      const color = COLORS[spriteCount % COLORS.length]
      spriteCount++

      // 创建 50x50 的矩形精灵图
      const sprite = engine.createRectSprite(50, 50, color[0], color[1], color[2], 255)

      // 随机位置 (在场景范围内)
      const offsetX = (Math.random() - 0.5) * 600
      const offsetY = (Math.random() - 0.5) * 400
      sprite.setPosition(offsetX, offsetY)

      // 设置初始 z-index
      engine.setSpriteZIndex(sprite, spriteCount)

      // 添加到场景
      engine.addToScene(sprite)

      // 保存到map
      sprites.set(sprite.id, sprite)

      // 创建控制卡片
      const card = createSpriteCard(sprite, color)
      spriteList.appendChild(card)
    }

    updateEmptyHint()
    console.log(`Batch created ${count} sprites`)
  })

  // 使用事件委托处理精灵图操作
  spriteList.addEventListener('click', (e) => {
    const target = e.target as HTMLElement

    // 删除按钮
    if (target.classList.contains('btn-delete')) {
      const spriteId = parseInt(target.dataset.spriteId || '0')
      removeSprite(spriteId)
      return
    }

    // 操作按钮
    if (target.classList.contains('btn-mini')) {
      const action = target.dataset.action
      const spriteId = parseInt(target.dataset.spriteId || '0')
      if (action) {
        handleSpriteAction(action, spriteId)
      }
    }
  })
}

// 启动
main().catch(console.error)
