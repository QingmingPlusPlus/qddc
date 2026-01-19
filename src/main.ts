import './style.css'
import init, { MemoryBlock } from '../pkg/qddc_wasm'

// 初始化 WASM 并设置 UI
async function main() {
  // 初始化 WASM 模块
  const wasm = await init()

  // 创建一个 16 字节的内存块
  const memoryBlock = new MemoryBlock(16)

  // 获取 WASM 内存视图
  const memory = wasm.memory

  // 创建 UI
  const app = document.querySelector<HTMLDivElement>('#app')!
  app.innerHTML = `
        <div class="container">
            <h1>WASM 内存操作演示</h1>
            <p class="description">Rust 端开辟了一块 ${memoryBlock.size()} 字节的内存</p>
            
            <div class="button-group">
                <button id="fillZeros" class="btn btn-blue">内存全部填 0</button>
                <button id="fillOnes" class="btn btn-red">内存全部填 1</button>
            </div>
            
            <div class="memory-display">
                <h2>内存内容 (十六进制)</h2>
                <pre id="memoryView" class="memory-view"></pre>
            </div>
            
            <div class="memory-display">
                <h2>内存内容 (二进制)</h2>
                <pre id="binaryView" class="binary-view"></pre>
            </div>
        </div>
    `

  // 更新内存显示
  function updateMemoryView() {
    const ptr = memoryBlock.data_ptr()
    const size = memoryBlock.size()
    const data = new Uint8Array(memory.buffer, ptr, size)

    // 十六进制显示
    const hexView = document.getElementById('memoryView')!
    const hexStr = Array.from(data)
      .map(b => b.toString(16).padStart(2, '0').toUpperCase())
      .join(' ')
    hexView.textContent = hexStr

    // 二进制显示
    const binaryView = document.getElementById('binaryView')!
    const binaryStr = Array.from(data)
      .map(b => b.toString(2).padStart(8, '0'))
      .join(' ')
    binaryView.textContent = binaryStr
  }

  // 初始显示
  updateMemoryView()

  // 绑定按钮事件
  document.getElementById('fillZeros')!.addEventListener('click', () => {
    memoryBlock.fill_zeros()
    updateMemoryView()
  })

  document.getElementById('fillOnes')!.addEventListener('click', () => {
    memoryBlock.fill_ones()
    updateMemoryView()
  })
}

main().catch(console.error)
