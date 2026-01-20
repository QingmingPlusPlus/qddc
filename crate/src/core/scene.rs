//! 场景模块
//!
//! 场景是最终显示内容的容器，管理精灵图的渲染。

use super::sampling::SamplingMethod;

/// 场景 - 最终显示内容的容器
///
/// 场景有固定的尺寸和像素缓冲区，可以添加多个精灵图并渲染到缓冲区。
#[derive(Debug)]
pub struct Scene {
    /// RGBA 像素数据
    data: Vec<u8>,
    /// 场景宽度
    width: u32,
    /// 场景高度
    height: u32,
    /// 添加到场景的精灵图 ID 列表
    sprite_ids: Vec<u32>,
    /// 背景颜色 RGBA
    background_color: [u8; 4],
    /// 采样方法
    sampling_method: SamplingMethod,
}

impl Scene {
    /// 创建新场景
    ///
    /// # Arguments
    /// * `width` - 场景宽度
    /// * `height` - 场景高度
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        let data = vec![0u8; size];

        Self {
            data,
            width,
            height,
            sprite_ids: Vec::new(),
            background_color: [0, 0, 0, 255], // 默认黑色背景
            sampling_method: SamplingMethod::default(),
        }
    }

    /// 获取采样方法
    pub fn sampling_method(&self) -> SamplingMethod {
        self.sampling_method
    }

    /// 设置采样方法
    pub fn set_sampling_method(&mut self, method: SamplingMethod) {
        self.sampling_method = method;
    }

    /// 获取场景宽度
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 获取场景高度
    pub fn height(&self) -> u32 {
        self.height
    }

    /// 设置背景颜色
    pub fn set_background_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.background_color = [r, g, b, a];
    }

    /// 添加精灵图到场景
    ///
    /// # Arguments
    /// * `sprite_id` - 精灵图 ID
    pub fn add_sprite(&mut self, sprite_id: u32) {
        if !self.sprite_ids.contains(&sprite_id) {
            self.sprite_ids.push(sprite_id);
        }
    }

    /// 从场景移除精灵图
    ///
    /// # Arguments
    /// * `sprite_id` - 精灵图 ID
    pub fn remove_sprite(&mut self, sprite_id: u32) {
        self.sprite_ids.retain(|&id| id != sprite_id);
    }

    /// 获取场景中的精灵图 ID 列表
    pub fn sprite_ids(&self) -> &[u32] {
        &self.sprite_ids
    }

    /// 清空场景缓冲区 (填充背景色)
    pub fn clear(&mut self) {
        for chunk in self.data.chunks_exact_mut(4) {
            chunk[0] = self.background_color[0];
            chunk[1] = self.background_color[1];
            chunk[2] = self.background_color[2];
            chunk[3] = self.background_color[3];
        }
    }

    /// 获取可变像素缓冲区引用
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// 获取只读像素缓冲区引用
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// 获取像素数据的指针 (用于 JS 端访问共享内存)
    pub fn data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// 获取数据长度
    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    /// 调整场景尺寸
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let new_size = (width * height * 4) as usize;
        self.data.resize(new_size, 0);
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scene() {
        let scene = Scene::new(800, 600);
        assert_eq!(scene.width(), 800);
        assert_eq!(scene.height(), 600);
        assert_eq!(scene.data_len(), 800 * 600 * 4);
    }

    #[test]
    fn test_add_remove_sprite() {
        let mut scene = Scene::new(100, 100);
        scene.add_sprite(1);
        scene.add_sprite(2);
        assert_eq!(scene.sprite_ids(), &[1, 2]);

        scene.remove_sprite(1);
        assert_eq!(scene.sprite_ids(), &[2]);
    }

    #[test]
    fn test_clear() {
        let mut scene = Scene::new(2, 2);
        scene.set_background_color(255, 0, 0, 255);
        scene.clear();

        let data = scene.data();
        assert_eq!(&data[0..4], &[255, 0, 0, 255]);
    }
}
