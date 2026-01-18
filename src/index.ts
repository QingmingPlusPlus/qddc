/**
 * QDDC - WebAssembly 2D Canvas 渲染库
 * 
 * 通过 Rust/WASM 控制内存，共享给 JS 端的 Canvas 进行高效 2D 图像渲染。
 */

// TODO: 导入 WASM 模块
// export { initWasm, WasmRenderer } from './wasm'

// 临时导出，稍后替换为实际 API
export const VERSION = '0.0.1'

/**
 * 初始化渲染器
 * @param canvas - 目标 Canvas 元素
 * @param options - 配置选项
 */
export async function createRenderer(
    _canvas: HTMLCanvasElement,
    _options?: RendererOptions
): Promise<Renderer> {
    // TODO: 实现 WASM 加载和初始化
    throw new Error('Not implemented yet')
}

/**
 * 渲染器配置选项
 */
export interface RendererOptions {
    /** 画布宽度 */
    width?: number
    /** 画布高度 */
    height?: number
}

/**
 * 渲染器接口
 */
export interface Renderer {
    /** 获取共享内存 buffer */
    getBuffer(): Uint8ClampedArray
    /** 将 buffer 渲染到 Canvas */
    render(): void
    /** 调整画布尺寸 */
    resize(width: number, height: number): void
    /** 销毁渲染器 */
    destroy(): void
}
