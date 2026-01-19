use wasm_bindgen::prelude::*;

/// 像素缓冲区 - 存储 RGBA 数据
/// 
/// 这个结构体持有一个 RGBA 格式的像素数组，可以直接与 JS 端的 Canvas ImageData 共享。
#[wasm_bindgen]
pub struct PixelBuffer {
    width: u32,
    height: u32,
    /// RGBA 像素数据，每个像素 4 字节
    data: Vec<u8>,
}

#[wasm_bindgen]
impl PixelBuffer {
    /// 创建新的像素缓冲区
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        let data = vec![0u8; size];
        Self { width, height, data }
    }

    /// 获取宽度
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 获取高度
    pub fn height(&self) -> u32 {
        self.height
    }

    /// 获取像素数据的指针（用于 JS 端访问共享内存）
    pub fn data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// 获取数据长度
    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    /// 调整缓冲区尺寸
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let new_size = (width * height * 4) as usize;
        self.data.resize(new_size, 0);
    }

    /// 清空缓冲区（填充指定颜色）
    pub fn clear(&mut self, r: u8, g: u8, b: u8, a: u8) {
        for chunk in self.data.chunks_exact_mut(4) {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = a;
        }
    }

    /// 设置单个像素颜色
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            self.data[idx] = r;
            self.data[idx + 1] = g;
            self.data[idx + 2] = b;
            self.data[idx + 3] = a;
        }
    }

    /// 填充矩形区域
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, r: u8, g: u8, b: u8, a: u8) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_pixel(x + dx, y + dy, r, g, b, a);
            }
        }
    }
}

/// WASM 模块初始化时调用
#[wasm_bindgen(start)]
pub fn init() {
    // 设置 panic hook 以便在控制台显示 Rust panic 信息
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// 简单内存块 - 演示 WASM 内存操作
/// 
/// 这个结构体持有一块固定大小的内存，可以通过方法全部置0或置1
#[wasm_bindgen]
pub struct MemoryBlock {
    /// 内存数据
    data: Vec<u8>,
}

#[wasm_bindgen]
impl MemoryBlock {
    /// 创建新的内存块
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize) -> Self {
        let data = vec![0u8; size];
        Self { data }
    }

    /// 获取内存大小
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// 获取内存数据的指针（用于 JS 端访问共享内存）
    pub fn data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// 将内存全部填充为 0
    pub fn fill_zeros(&mut self) {
        for byte in self.data.iter_mut() {
            *byte = 0;
        }
    }

    /// 将内存全部填充为 1 (0xFF)
    pub fn fill_ones(&mut self) {
        for byte in self.data.iter_mut() {
            *byte = 0xFF;
        }
    }
}
