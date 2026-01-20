import './style.css'
import { Engine, Sprite } from './engine'

// 状态
let engine: Engine | null = null
let currentSprite: Sprite | null = null
let spriteCount = 0

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

/**
 * 更新精灵图状态显示
 */
function updateSpriteInfo() {
  const infoEl = document.getElementById('spriteInfo')!

  if (!currentSprite) {
    infoEl.textContent = '暂无精灵图'
    return
  }

  infoEl.textContent = `ID: ${currentSprite.id}
位置: (${currentSprite.position.x.toFixed(1)}, ${currentSprite.position.y.toFixed(1)})
旋转: ${(currentSprite.rotation * 180 / Math.PI).toFixed(1)}°
缩放: (${currentSprite.scale.x.toFixed(2)}, ${currentSprite.scale.y.toFixed(2)})`
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

  // 绑定按钮事件
  bindButtonEvents()

  console.log('QDDC Engine initialized!')
}

/**
 * 绑定按钮事件
 */
function bindButtonEvents() {
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

    // 设为当前精灵图
    currentSprite = sprite
    updateSpriteInfo()

    console.log(`Created sprite ${sprite.id} at (${offsetX.toFixed(1)}, ${offsetY.toFixed(1)})`)
  })

  // 删除精灵图
  document.getElementById('removeSprite')!.addEventListener('click', () => {
    if (!engine || !currentSprite) return

    engine.removeSprite(currentSprite)
    console.log(`Removed sprite ${currentSprite.id}`)
    currentSprite = null
    updateSpriteInfo()
  })

  // 平移
  const TRANSLATE_STEP = 20

  document.getElementById('translateLeft')!.addEventListener('click', () => {
    if (!currentSprite) return
    currentSprite.translate(-TRANSLATE_STEP, 0)
    updateSpriteInfo()
  })

  document.getElementById('translateRight')!.addEventListener('click', () => {
    if (!currentSprite) return
    currentSprite.translate(TRANSLATE_STEP, 0)
    updateSpriteInfo()
  })

  document.getElementById('translateUp')!.addEventListener('click', () => {
    if (!currentSprite) return
    currentSprite.translate(0, -TRANSLATE_STEP)
    updateSpriteInfo()
  })

  document.getElementById('translateDown')!.addEventListener('click', () => {
    if (!currentSprite) return
    currentSprite.translate(0, TRANSLATE_STEP)
    updateSpriteInfo()
  })

  // 旋转
  const ROTATE_STEP = Math.PI / 12  // 15 度

  document.getElementById('rotateCCW')!.addEventListener('click', () => {
    if (!currentSprite) return
    currentSprite.rotate(-ROTATE_STEP)
    updateSpriteInfo()
  })

  document.getElementById('rotateCW')!.addEventListener('click', () => {
    if (!currentSprite) return
    currentSprite.rotate(ROTATE_STEP)
    updateSpriteInfo()
  })

  // 缩放
  const SCALE_STEP = 1.2

  document.getElementById('scaleUp')!.addEventListener('click', () => {
    if (!currentSprite) return
    currentSprite.scaleBy(SCALE_STEP, SCALE_STEP)
    updateSpriteInfo()
  })

  document.getElementById('scaleDown')!.addEventListener('click', () => {
    if (!currentSprite) return
    currentSprite.scaleBy(1 / SCALE_STEP, 1 / SCALE_STEP)
    updateSpriteInfo()
  })
}

// 启动
main().catch(console.error)
