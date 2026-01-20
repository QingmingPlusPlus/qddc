//! ECS 世界管理器
//!
//! 管理所有精灵图和场景，提供暴露给 JS 的接口。

use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use super::scene::Scene;
use super::sprite::Sprite;

/// ECS 世界管理器
///
/// 管理所有实体（精灵图）和场景，提供统一的操作接口。
#[wasm_bindgen]
pub struct World {
    /// 下一个可用的精灵图 ID
    next_id: u32,
    /// 精灵图存储
    sprites: HashMap<u32, Sprite>,
    /// 场景
    scene: Scene,
}

#[wasm_bindgen]
impl World {
    /// 创建新的世界
    ///
    /// # Arguments
    /// * `width` - 场景宽度
    /// * `height` - 场景高度
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            next_id: 1,
            sprites: HashMap::new(),
            scene: Scene::new(width, height),
        }
    }

    /// 创建精灵图
    ///
    /// # Arguments
    /// * `data` - RGBA 像素数据
    /// * `width` - 图像宽度
    /// * `height` - 图像高度
    ///
    /// # Returns
    /// 精灵图 ID
    pub fn create_sprite(&mut self, data: &[u8], width: u32, height: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let sprite = Sprite::new(id, data.to_vec(), width, height);
        self.sprites.insert(id, sprite);

        id
    }

    /// 创建矩形精灵图
    ///
    /// # Arguments
    /// * `width` - 矩形宽度
    /// * `height` - 矩形高度
    /// * `r`, `g`, `b`, `a` - RGBA 颜色
    ///
    /// # Returns
    /// 精灵图 ID
    pub fn create_rect_sprite(
        &mut self,
        width: u32,
        height: u32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> u32 {
        let size = (width * height * 4) as usize;
        let mut data = vec![0u8; size];

        for i in 0..(width * height) as usize {
            data[i * 4] = r;
            data[i * 4 + 1] = g;
            data[i * 4 + 2] = b;
            data[i * 4 + 3] = a;
        }

        self.create_sprite(&data, width, height)
    }

    /// 移除精灵图
    ///
    /// # Arguments
    /// * `id` - 精灵图 ID
    pub fn remove_sprite(&mut self, id: u32) {
        self.sprites.remove(&id);
        self.scene.remove_sprite(id);
    }

    /// 平移精灵图
    ///
    /// # Arguments
    /// * `id` - 精灵图 ID
    /// * `dx` - X 方向平移量
    /// * `dy` - Y 方向平移量
    pub fn translate_sprite(&mut self, id: u32, dx: f32, dy: f32) {
        if let Some(sprite) = self.sprites.get_mut(&id) {
            sprite.translate(dx, dy);
        }
    }

    /// 设置精灵图位置
    ///
    /// # Arguments
    /// * `id` - 精灵图 ID
    /// * `x` - X 坐标
    /// * `y` - Y 坐标
    pub fn set_sprite_position(&mut self, id: u32, x: f32, y: f32) {
        if let Some(sprite) = self.sprites.get_mut(&id) {
            sprite.set_position(x, y);
        }
    }

    /// 旋转精灵图
    ///
    /// # Arguments
    /// * `id` - 精灵图 ID
    /// * `angle` - 旋转角度 (弧度)
    pub fn rotate_sprite(&mut self, id: u32, angle: f32) {
        if let Some(sprite) = self.sprites.get_mut(&id) {
            sprite.rotate(angle);
        }
    }

    /// 设置精灵图旋转角度
    ///
    /// # Arguments
    /// * `id` - 精灵图 ID
    /// * `angle` - 角度 (弧度)
    pub fn set_sprite_rotation(&mut self, id: u32, angle: f32) {
        if let Some(sprite) = self.sprites.get_mut(&id) {
            sprite.set_rotation(angle);
        }
    }

    /// 缩放精灵图
    ///
    /// # Arguments
    /// * `id` - 精灵图 ID
    /// * `sx` - X 方向缩放因子
    /// * `sy` - Y 方向缩放因子
    pub fn scale_sprite(&mut self, id: u32, sx: f32, sy: f32) {
        if let Some(sprite) = self.sprites.get_mut(&id) {
            sprite.scale_by(sx, sy);
        }
    }

    /// 设置精灵图缩放
    ///
    /// # Arguments
    /// * `id` - 精灵图 ID
    /// * `sx` - X 方向缩放因子
    /// * `sy` - Y 方向缩放因子
    pub fn set_sprite_scale(&mut self, id: u32, sx: f32, sy: f32) {
        if let Some(sprite) = self.sprites.get_mut(&id) {
            sprite.set_scale(sx, sy);
        }
    }

    /// 设置精灵图 z-index
    ///
    /// # Arguments
    /// * `id` - 精灵图 ID
    /// * `zindex` - z-index 值
    pub fn set_sprite_zindex(&mut self, id: u32, zindex: i32) {
        if let Some(sprite) = self.sprites.get_mut(&id) {
            sprite.set_zindex(zindex);
        }
    }

    /// 添加精灵图到场景
    ///
    /// # Arguments
    /// * `sprite_id` - 精灵图 ID
    pub fn add_to_scene(&mut self, sprite_id: u32) {
        if self.sprites.contains_key(&sprite_id) {
            self.scene.add_sprite(sprite_id);
        }
    }

    /// 从场景移除精灵图
    ///
    /// # Arguments
    /// * `sprite_id` - 精灵图 ID
    pub fn remove_from_scene(&mut self, sprite_id: u32) {
        self.scene.remove_sprite(sprite_id);
    }

    /// 设置场景背景色
    pub fn set_background_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.scene.set_background_color(r, g, b, a);
    }

    /// 渲染一帧
    ///
    /// 清空场景缓冲区，按 z-index 排序精灵图，然后逐个渲染。
    pub fn render(&mut self) {
        // 清空场景
        self.scene.clear();

        // 收集需要渲染的精灵图，按 z-index 排序
        let mut to_render: Vec<u32> = self
            .scene
            .sprite_ids()
            .iter()
            .filter(|id| self.sprites.contains_key(id))
            .cloned()
            .collect();

        to_render.sort_by_key(|id| self.sprites.get(id).map(|s| s.zindex()).unwrap_or(0));

        // 渲染每个精灵图
        let width = self.scene.width();
        let height = self.scene.height();

        for sprite_id in to_render {
            if let Some(sprite) = self.sprites.get(&sprite_id) {
                // 由于借用检查，我们需要先克隆精灵图或分离数据
                let sprite_clone = sprite.clone();
                sprite_clone.render_to(self.scene.data_mut(), width, height);
            }
        }
    }

    /// 获取场景数据指针 (供 JS 读取渲染结果)
    pub fn scene_data_ptr(&self) -> *const u8 {
        self.scene.data_ptr()
    }

    /// 获取场景数据长度
    pub fn scene_data_len(&self) -> usize {
        self.scene.data_len()
    }

    /// 获取场景宽度
    pub fn scene_width(&self) -> u32 {
        self.scene.width()
    }

    /// 获取场景高度
    pub fn scene_height(&self) -> u32 {
        self.scene.height()
    }

    /// 调整场景尺寸
    pub fn resize_scene(&mut self, width: u32, height: u32) {
        self.scene.resize(width, height);
    }

    /// 获取精灵图位置
    pub fn get_sprite_position(&self, id: u32) -> Option<Vec<f32>> {
        self.sprites.get(&id).map(|s| {
            let pos = s.position();
            vec![pos.0, pos.1]
        })
    }

    /// 获取精灵图旋转角度
    pub fn get_sprite_rotation(&self, id: u32) -> Option<f32> {
        self.sprites.get(&id).map(|s| s.rotation())
    }

    /// 获取精灵图缩放
    pub fn get_sprite_scale(&self, id: u32) -> Option<Vec<f32>> {
        self.sprites.get(&id).map(|s| {
            let scale = s.scale();
            vec![scale.0, scale.1]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_world() {
        let world = World::new(800, 600);
        assert_eq!(world.scene_width(), 800);
        assert_eq!(world.scene_height(), 600);
    }

    #[test]
    fn test_create_sprite() {
        let mut world = World::new(100, 100);
        let id = world.create_rect_sprite(10, 10, 255, 0, 0, 255);
        assert_eq!(id, 1);

        let id2 = world.create_rect_sprite(10, 10, 0, 255, 0, 255);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_sprite_transforms() {
        let mut world = World::new(100, 100);
        let id = world.create_rect_sprite(10, 10, 255, 0, 0, 255);

        world.translate_sprite(id, 10.0, 20.0);
        let pos = world.get_sprite_position(id).unwrap();
        assert_eq!(pos, vec![10.0, 20.0]);
    }

    #[test]
    fn test_scene_management() {
        let mut world = World::new(100, 100);
        let id = world.create_rect_sprite(10, 10, 255, 0, 0, 255);

        world.add_to_scene(id);
        world.render();

        // 验证渲染成功（不崩溃即可）
        assert!(world.scene_data_len() > 0);
    }
}
