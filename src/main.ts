import './style.css'
import { Engine, Sprite, SamplingMethod } from './engine'

// 状态
let engine: Engine | null = null
let spriteCount = 0
const sprites: Map<number, Sprite> = new Map()

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
      <span class="info-label">缩放:</span> <span class="scale-x">${sprite.scale.x.toFixed(2)}</span>, <span class="scale-y">${sprite.scale.y.toFixed(2)}</span>
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

  if (posX) posX.textContent = sprite.position.x.toFixed(1)
  if (posY) posY.textContent = sprite.position.y.toFixed(1)
  if (rotation) rotation.textContent = (sprite.rotation * 180 / Math.PI).toFixed(1)
  if (scaleX) scaleX.textContent = sprite.scale.x.toFixed(2)
  if (scaleY) scaleY.textContent = sprite.scale.y.toFixed(2)
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
  if (!sprite) return

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

  // 渲染循环
  function gameLoop() {
    if (engine) {
      engine.render()
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

  // 采样方法选择器
  const samplingSelect = document.getElementById('samplingMethod') as HTMLSelectElement
  samplingSelect.addEventListener('change', () => {
    if (!engine) return
    const method = samplingSelect.value as SamplingMethod
    engine.setSamplingMethod(method)
    console.log(`Sampling method changed to: ${method}`)
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

    // 添加到场景
    engine.addToScene(sprite)

    // 保存到map
    sprites.set(sprite.id, sprite)

    // 创建控制卡片
    const card = createSpriteCard(sprite, color)
    spriteList.appendChild(card)

    updateEmptyHint()

    console.log(`Created sprite ${sprite.id} at (${offsetX.toFixed(1)}, ${offsetY.toFixed(1)})`)
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
