import { defineConfig } from 'vite'
import { resolve } from 'path'

export default defineConfig({
  build: {
    lib: {
      // 库的入口文件
      entry: resolve(__dirname, 'src/index.ts'),
      // UMD 格式下的全局变量名
      name: 'Qddc',
      // 输出文件名
      fileName: 'qddc',
      // 输出格式：ESM 和 UMD
      formats: ['es', 'umd'],
    },
    rollupOptions: {
      // 外部依赖（不打包进库的模块）
      external: [],
      output: {
        globals: {},
      },
    },
    // 输出目录
    outDir: 'dist',
    // 生成 sourcemap
    sourcemap: true,
  },
  // 开发服务器配置
  server: {
    // 支持 SharedArrayBuffer（跨域隔离）
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
})
