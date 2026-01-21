/**
 * JS 端精灵图对象
 *
 * 与 WASM 精灵图 ID 对应，维护本地状态。
 * WASM 端只存储 id、data、size、position、zindex。
 * JS 端存储变换状态（旋转角度、缩放因子）用于累积变换。
 */
export class Sprite {
    /** 精灵图 ID (在 WASM 端分配，即数组索引) */
    readonly id: number

    /** 位置 (几何中心) */
    position: { x: number; y: number }

    /** 旋转角度 (弧度) - JS端累积值 */
    rotation: number

    /** 缩放因子 - JS端累积值 */
    scale: { x: number; y: number }

    /** Z层级 */
    zindex: number

    /** 引用的 World 更新函数 */
    private _updateFn: ((sprite: Sprite) => void) | null = null

    /** 是否需要同步变换到WASM */
    private _transformDirty: boolean = false

    constructor(id: number) {
        this.id = id
        this.position = { x: 0, y: 0 }
        this.rotation = 0
        this.scale = { x: 1, y: 1 }
        this.zindex = 0
    }

    /**
     * 设置更新回调 (由 Engine 内部调用)
     */
    _setUpdateFn(fn: (sprite: Sprite) => void) {
        this._updateFn = fn
    }

    /**
     * 检查变换是否需要同步
     */
    _isTransformDirty(): boolean {
        return this._transformDirty
    }

    /**
     * 清除变换脏标记
     */
    _clearTransformDirty() {
        this._transformDirty = false
    }

    /**
     * 平移精灵图
     * @param dx X 方向平移量
     * @param dy Y 方向平移量
     */
    translate(dx: number, dy: number) {
        this.position.x += dx
        this.position.y += dy
        this._notifyUpdate()
    }

    /**
     * 设置位置
     * @param x X 坐标
     * @param y Y 坐标
     */
    setPosition(x: number, y: number) {
        this.position.x = x
        this.position.y = y
        this._notifyUpdate()
    }

    /**
     * 旋转精灵图
     * @param angle 旋转角度 (弧度)
     */
    rotate(angle: number) {
        this.rotation += angle
        this._transformDirty = true
        this._notifyUpdate()
    }

    /**
     * 设置旋转角度
     * @param angle 角度 (弧度)
     */
    setRotation(angle: number) {
        this.rotation = angle
        this._transformDirty = true
        this._notifyUpdate()
    }

    /**
     * 设置缩放
     * @param sx X 方向缩放因子
     * @param sy Y 方向缩放因子
     */
    setScale(sx: number, sy: number) {
        this.scale.x = sx
        this.scale.y = sy
        this._transformDirty = true
        this._notifyUpdate()
    }

    /**
     * 缩放精灵图 (累乘)
     * @param sx X 方向缩放因子
     * @param sy Y 方向缩放因子
     */
    scaleBy(sx: number, sy: number) {
        this.scale.x *= sx
        this.scale.y *= sy
        this._transformDirty = true
        this._notifyUpdate()
    }

    /**
     * 设置 z-index
     * @param zindex 新的 z-index 值
     */
    setZIndex(zindex: number) {
        this.zindex = zindex
        this._notifyUpdate()
    }

    /**
     * 重置变换状态
     */
    resetTransform() {
        this.rotation = 0
        this.scale = { x: 1, y: 1 }
        this._transformDirty = true
        this._notifyUpdate()
    }

    private _notifyUpdate() {
        if (this._updateFn) {
            this._updateFn(this)
        }
    }
}
