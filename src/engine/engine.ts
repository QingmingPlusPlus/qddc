import init, { World } from '../../pkg/qddc_wasm'
import { Sprite } from './sprite'

/**
 * 采样方法类型
 */
export type SamplingMethod = 'nearest' | 'bilinear' | 'supersampling'

/**
 * 性能计时数据
 */
export interface PerformanceMetrics {
    /** WASM 渲染时间 (ms) */
    wasmRender: number
    /** 内存读取时间 (ms) */
    memoryRead: number
    /** ImageData 复制时间 (ms) */
    imageCopy: number
    /** Canvas 绘制时间 (ms) */
    canvasDraw: number
    /** 总渲染时间 (ms) */
    total: number
    /** 精灵数量 */
    spriteCount: number
}

/**
 * 2D 渲染引擎
 *
 * 封装 WASM World，提供精灵图管理和渲染功能。
 */
export class Engine {
    private world: World
    private sprites: Map<number, Sprite>
    private ctx: CanvasRenderingContext2D
    private imageData: ImageData
    private wasmMemory: WebAssembly.Memory

    private constructor(
        world: World,
        ctx: CanvasRenderingContext2D,
        wasmMemory: WebAssembly.Memory
    ) {
        this.world = world
        this.sprites = new Map()
        this.ctx = ctx
        this.wasmMemory = wasmMemory

        // 创建 ImageData 用于渲染
        this.imageData = ctx.createImageData(
            world.scene_width(),
            world.scene_height()
        )
    }

    /**
     * 创建引擎实例
     * @param canvas 目标 Canvas 元素
     */
    static async create(canvas: HTMLCanvasElement): Promise<Engine> {
        // 初始化 WASM
        const wasm = await init()

        const ctx = canvas.getContext('2d')
        if (!ctx) {
            throw new Error('Failed to get 2d context')
        }

        // 创建 World
        const world = new World(canvas.width, canvas.height)

        return new Engine(world, ctx, wasm.memory)
    }

    /**
     * 创建精灵图
     * @param imageData RGBA 像素数据
     * @param width 图像宽度
     * @param height 图像高度
     */
    createSprite(imageData: Uint8Array, width: number, height: number): Sprite {
        const id = this.world.create_sprite(imageData, width, height)
        const sprite = new Sprite(id)

        // 设置更新回调
        sprite._setUpdateFn((s) => this._syncSpriteToWasm(s))

        this.sprites.set(id, sprite)
        return sprite
    }

    /**
     * 创建矩形精灵图
     * @param width 矩形宽度
     * @param height 矩形高度
     * @param r 红色分量
     * @param g 绿色分量
     * @param b 蓝色分量
     * @param a 透明度
     */
    createRectSprite(
        width: number,
        height: number,
        r: number,
        g: number,
        b: number,
        a: number = 255
    ): Sprite {
        const id = this.world.create_rect_sprite(width, height, r, g, b, a)
        const sprite = new Sprite(id)

        // 设置更新回调
        sprite._setUpdateFn((s) => this._syncSpriteToWasm(s))

        this.sprites.set(id, sprite)
        return sprite
    }

    /**
     * 移除精灵图
     * @param sprite 精灵图对象
     */
    removeSprite(sprite: Sprite) {
        this.world.remove_sprite(sprite.id)
        this.sprites.delete(sprite.id)
    }

    /**
     * 添加精灵图到场景
     * @param sprite 精灵图对象
     */
    addToScene(sprite: Sprite) {
        this.world.add_to_scene(sprite.id)
    }

    /**
     * 从场景移除精灵图
     * @param sprite 精灵图对象
     */
    removeFromScene(sprite: Sprite) {
        this.world.remove_from_scene(sprite.id)
    }

    /**
     * 设置背景色
     */
    setBackgroundColor(r: number, g: number, b: number, a: number = 255) {
        this.world.set_background_color(r, g, b, a)
    }

    /**
     * 设置采样方法
     * @param method 采样方法: 'nearest' | 'bilinear' | 'supersampling'
     */
    setSamplingMethod(method: SamplingMethod) {
        const methodMap: Record<SamplingMethod, number> = {
            'nearest': 0,
            'bilinear': 1,
            'supersampling': 2,
        }
        this.world.set_sampling_method(methodMap[method])
    }

    /**
     * 获取当前采样方法
     */
    getSamplingMethod(): SamplingMethod {
        const value = this.world.get_sampling_method()
        const methods: SamplingMethod[] = ['nearest', 'bilinear', 'supersampling']
        return methods[value] || 'nearest'
    }

    /**
     * 设置精灵图 z-index
     * @param sprite 精灵图对象
     * @param zindex z-index 值
     */
    setSpriteZIndex(sprite: Sprite, zindex: number) {
        sprite.zindex = zindex
        this.world.set_sprite_zindex(sprite.id, zindex)
    }

    /**
     * 获取精灵图 z-index
     * @param sprite 精灵图对象
     */
    getSpriteZIndex(sprite: Sprite): number {
        return this.world.get_sprite_zindex(sprite.id)
    }

    /**
     * 重置精灵图变换
     * @param sprite 精灵图对象
     */
    resetSpriteTransform(sprite: Sprite) {
        sprite.resetTransform()
        this.world.reset_sprite_transform(sprite.id)
    }

    /**
     * 渲染一帧到 Canvas
     */
    render() {
        // 调用 WASM 渲染
        this.world.render()

        // 从 WASM 内存读取渲染结果
        const ptr = this.world.scene_data_ptr()
        const len = this.world.scene_data_len()
        const data = new Uint8ClampedArray(this.wasmMemory.buffer, ptr, len)

        // 复制到 ImageData
        this.imageData.data.set(data)

        // 绘制到 Canvas
        this.ctx.putImageData(this.imageData, 0, 0)
    }

    /**
     * 渲染一帧并返回性能计时数据
     */
    renderWithTiming(): PerformanceMetrics {
        const startTotal = performance.now()

        // 1. WASM 渲染
        const startWasm = performance.now()
        this.world.render()
        const wasmRender = performance.now() - startWasm

        // 2. 从 WASM 内存读取渲染结果
        const startMemory = performance.now()
        const ptr = this.world.scene_data_ptr()
        const len = this.world.scene_data_len()
        const data = new Uint8ClampedArray(this.wasmMemory.buffer, ptr, len)
        const memoryRead = performance.now() - startMemory

        // 3. 复制到 ImageData
        const startCopy = performance.now()
        this.imageData.data.set(data)
        const imageCopy = performance.now() - startCopy

        // 4. 绘制到 Canvas
        const startDraw = performance.now()
        this.ctx.putImageData(this.imageData, 0, 0)
        const canvasDraw = performance.now() - startDraw

        const total = performance.now() - startTotal

        return {
            wasmRender,
            memoryRead,
            imageCopy,
            canvasDraw,
            total,
            spriteCount: this.sprites.size
        }
    }

    /**
     * 调整场景尺寸
     */
    resize(width: number, height: number) {
        this.world.resize_scene(width, height)
        this.imageData = this.ctx.createImageData(width, height)
    }

    /**
     * 获取精灵图
     */
    getSprite(id: number): Sprite | undefined {
        return this.sprites.get(id)
    }

    /**
     * 同步精灵图状态到 WASM
     */
    private _syncSpriteToWasm(sprite: Sprite) {
        // 同步位置
        this.world.set_sprite_position(sprite.id, sprite.position.x, sprite.position.y)

        // 同步 z-index
        this.world.set_sprite_zindex(sprite.id, sprite.zindex)

        // 如果变换有变化，应用组合变换
        if (sprite._isTransformDirty()) {
            this.world.apply_sprite_transform(sprite.id, sprite.rotation, sprite.scale.x, sprite.scale.y)
            sprite._clearTransformDirty()
        }
    }
}
