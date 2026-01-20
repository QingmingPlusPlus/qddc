/**
 * JS 端精灵图对象
 *
 * 与 WASM 精灵图 ID 对应，维护本地状态。
 */
export class Sprite {
    /** 精灵图 ID (在 WASM 端分配) */
    readonly id: number

    /** 位置 (几何中心) */
    position: { x: number; y: number }

    /** 旋转角度 (弧度) */
    rotation: number

    /** 缩放因子 */
    scale: { x: number; y: number }

    /** 引用的 World 更新函数 */
    private _updateFn: ((sprite: Sprite) => void) | null = null

    constructor(id: number) {
        this.id = id
        this.position = { x: 0, y: 0 }
        this.rotation = 0
        this.scale = { x: 1, y: 1 }
    }

    /**
     * 设置更新回调 (由 Engine 内部调用)
     */
    _setUpdateFn(fn: (sprite: Sprite) => void) {
        this._updateFn = fn
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
        this._notifyUpdate()
    }

    /**
     * 设置旋转角度
     * @param angle 角度 (弧度)
     */
    setRotation(angle: number) {
        this.rotation = angle
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
        this._notifyUpdate()
    }

    private _notifyUpdate() {
        if (this._updateFn) {
            this._updateFn(this)
        }
    }
}
