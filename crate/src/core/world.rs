//! ECS 世界管理器
//!
//! 纯数据导向的ECS架构，所有数据存储在数组中，ID即索引。

use wasm_bindgen::prelude::*;

use super::sampling::{sample_bilinear, sample_nearest, sample_supersampling, SamplingMethod};
use crate::math::Matrix3x3;

/// 精灵图存储 - 各属性分离为独立数组
pub struct SpriteStore {
    /// 原始像素数据 (只读，用于变换)
    original_data: Vec<Vec<u8>>,
    /// 显示像素数据 (变换结果)
    display_data: Vec<Vec<u8>>,
    /// 原始宽度
    original_widths: Vec<u32>,
    /// 原始高度
    original_heights: Vec<u32>,
    /// 显示宽度 (变换后)
    display_widths: Vec<u32>,
    /// 显示高度 (变换后)
    display_heights: Vec<u32>,
    /// X 坐标
    positions_x: Vec<f32>,
    /// Y 坐标
    positions_y: Vec<f32>,
    /// Z 层级
    zindexes: Vec<i32>,
    /// 是否活跃 (用于删除标记)
    active: Vec<bool>,
}

impl SpriteStore {
    fn new() -> Self {
        Self {
            original_data: Vec::new(),
            display_data: Vec::new(),
            original_widths: Vec::new(),
            original_heights: Vec::new(),
            display_widths: Vec::new(),
            display_heights: Vec::new(),
            positions_x: Vec::new(),
            positions_y: Vec::new(),
            zindexes: Vec::new(),
            active: Vec::new(),
        }
    }

    /// 添加新精灵图，返回ID (索引)
    fn add(&mut self, data: Vec<u8>, width: u32, height: u32) -> u32 {
        let id = self.original_data.len() as u32;
        self.original_data.push(data.clone());
        self.display_data.push(data);
        self.original_widths.push(width);
        self.original_heights.push(height);
        self.display_widths.push(width);
        self.display_heights.push(height);
        self.positions_x.push(0.0);
        self.positions_y.push(0.0);
        self.zindexes.push(0);
        self.active.push(true);
        id
    }

    /// 移除精灵图 (标记为非活跃)
    fn remove(&mut self, id: u32) {
        let idx = id as usize;
        if idx < self.active.len() {
            self.active[idx] = false;
        }
    }

    /// 检查精灵图是否存在且活跃
    fn is_active(&self, id: u32) -> bool {
        let idx = id as usize;
        idx < self.active.len() && self.active[idx]
    }
}

/// 场景存储 - 各属性分离为独立数组
pub struct SceneStore {
    /// 像素缓冲数据
    data: Vec<Vec<u8>>,
    /// 宽度
    widths: Vec<u32>,
    /// 高度
    heights: Vec<u32>,
    /// Z 层级
    zindexes: Vec<i32>,
    /// 背景色
    background_colors: Vec<[u8; 4]>,
    /// 包含的精灵图ID列表
    sprite_ids: Vec<Vec<u32>>,
    /// 采样方法
    sampling_methods: Vec<SamplingMethod>,
    /// 是否活跃
    active: Vec<bool>,
}

impl SceneStore {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            widths: Vec::new(),
            heights: Vec::new(),
            zindexes: Vec::new(),
            background_colors: Vec::new(),
            sprite_ids: Vec::new(),
            sampling_methods: Vec::new(),
            active: Vec::new(),
        }
    }

    /// 添加新场景，返回ID (索引)
    fn add(&mut self, width: u32, height: u32) -> u32 {
        let id = self.data.len() as u32;
        let size = (width * height * 4) as usize;
        self.data.push(vec![0u8; size]);
        self.widths.push(width);
        self.heights.push(height);
        self.zindexes.push(0);
        self.background_colors.push([0, 0, 0, 255]);
        self.sprite_ids.push(Vec::new());
        self.sampling_methods.push(SamplingMethod::default());
        self.active.push(true);
        id
    }

    /// 检查场景是否存在且活跃
    fn is_active(&self, id: u32) -> bool {
        let idx = id as usize;
        idx < self.active.len() && self.active[idx]
    }
}

/// ECS 世界管理器
///
/// 管理精灵图和场景的数组存储。
#[wasm_bindgen]
pub struct World {
    /// 精灵图存储
    sprites: SpriteStore,
    /// 场景存储
    scenes: SceneStore,
    /// 默认场景ID (保持向后兼容)
    default_scene: u32,
}

#[wasm_bindgen]
impl World {
    /// 创建新的世界
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        let mut world = Self {
            sprites: SpriteStore::new(),
            scenes: SceneStore::new(),
            default_scene: 0,
        };
        // 创建默认场景
        world.default_scene = world.scenes.add(width, height);
        world
    }

    // ========== 精灵图操作 ==========

    /// 创建精灵图
    pub fn create_sprite(&mut self, data: &[u8], width: u32, height: u32) -> u32 {
        self.sprites.add(data.to_vec(), width, height)
    }

    /// 创建矩形精灵图
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

        self.sprites.add(data, width, height)
    }

    /// 移除精灵图
    pub fn remove_sprite(&mut self, id: u32) {
        self.sprites.remove(id);
        // 从所有场景中移除
        for sprite_ids in &mut self.scenes.sprite_ids {
            sprite_ids.retain(|&sid| sid != id);
        }
    }

    /// 设置精灵图位置
    pub fn set_sprite_position(&mut self, id: u32, x: f32, y: f32) {
        let idx = id as usize;
        if self.sprites.is_active(id) {
            self.sprites.positions_x[idx] = x;
            self.sprites.positions_y[idx] = y;
        }
    }

    /// 获取精灵图位置
    pub fn get_sprite_position(&self, id: u32) -> Option<Vec<f32>> {
        if self.sprites.is_active(id) {
            let idx = id as usize;
            Some(vec![
                self.sprites.positions_x[idx],
                self.sprites.positions_y[idx],
            ])
        } else {
            None
        }
    }

    /// 平移精灵图
    pub fn translate_sprite(&mut self, id: u32, dx: f32, dy: f32) {
        let idx = id as usize;
        if self.sprites.is_active(id) {
            self.sprites.positions_x[idx] += dx;
            self.sprites.positions_y[idx] += dy;
        }
    }

    /// 设置精灵图 z-index
    pub fn set_sprite_zindex(&mut self, id: u32, zindex: i32) {
        let idx = id as usize;
        if self.sprites.is_active(id) {
            self.sprites.zindexes[idx] = zindex;
        }
    }

    /// 获取精灵图 z-index
    pub fn get_sprite_zindex(&self, id: u32) -> i32 {
        let idx = id as usize;
        if self.sprites.is_active(id) {
            self.sprites.zindexes[idx]
        } else {
            0
        }
    }

    /// 应用旋转变换到精灵图
    ///
    /// 在原始数据的副本上应用旋转，结果覆盖显示数据。
    pub fn apply_sprite_rotation(&mut self, id: u32, angle: f32) {
        let idx = id as usize;
        if !self.sprites.is_active(id) {
            return;
        }

        let orig_width = self.sprites.original_widths[idx];
        let orig_height = self.sprites.original_heights[idx];
        let orig_data = &self.sprites.original_data[idx];

        // 计算旋转后的边界框
        let cos_a = angle.cos().abs();
        let sin_a = angle.sin().abs();
        let new_width =
            (orig_width as f32 * cos_a + orig_height as f32 * sin_a).ceil() as u32;
        let new_height =
            (orig_width as f32 * sin_a + orig_height as f32 * cos_a).ceil() as u32;

        let new_size = (new_width * new_height * 4) as usize;
        let mut new_data = vec![0u8; new_size];

        // 创建旋转矩阵及其逆矩阵
        let rotation = Matrix3x3::rotation(-angle);
        let inverse = rotation.inverse().unwrap_or_else(Matrix3x3::identity);

        let orig_half_w = orig_width as f32 / 2.0;
        let orig_half_h = orig_height as f32 / 2.0;
        let new_half_w = new_width as f32 / 2.0;
        let new_half_h = new_height as f32 / 2.0;

        // 逐像素变换
        for ty in 0..new_height {
            for tx in 0..new_width {
                let dst_x = tx as f32 - new_half_w;
                let dst_y = ty as f32 - new_half_h;

                let (src_x, src_y) = inverse.transform_point(dst_x, dst_y);
                let src_px = src_x + orig_half_w;
                let src_py = src_y + orig_half_h;

                if let Some(color) =
                    sample_bilinear(orig_data, orig_width, orig_height, src_px, src_py)
                {
                    let dst_idx = ((ty * new_width + tx) * 4) as usize;
                    new_data[dst_idx] = color[0];
                    new_data[dst_idx + 1] = color[1];
                    new_data[dst_idx + 2] = color[2];
                    new_data[dst_idx + 3] = color[3];
                }
            }
        }

        self.sprites.display_data[idx] = new_data;
        self.sprites.display_widths[idx] = new_width;
        self.sprites.display_heights[idx] = new_height;
    }

    /// 应用缩放变换到精灵图
    ///
    /// 在原始数据的副本上应用缩放，结果覆盖显示数据。
    pub fn apply_sprite_scale(&mut self, id: u32, sx: f32, sy: f32) {
        let idx = id as usize;
        if !self.sprites.is_active(id) || sx.abs() < 0.001 || sy.abs() < 0.001 {
            return;
        }

        let orig_width = self.sprites.original_widths[idx];
        let orig_height = self.sprites.original_heights[idx];
        let orig_data = &self.sprites.original_data[idx];

        let new_width = (orig_width as f32 * sx.abs()).round() as u32;
        let new_height = (orig_height as f32 * sy.abs()).round() as u32;

        if new_width == 0 || new_height == 0 {
            return;
        }

        let new_size = (new_width * new_height * 4) as usize;
        let mut new_data = vec![0u8; new_size];

        // 逐像素采样
        for ty in 0..new_height {
            for tx in 0..new_width {
                let src_px = (tx as f32 + 0.5) / sx.abs();
                let src_py = (ty as f32 + 0.5) / sy.abs();

                if let Some(color) =
                    sample_bilinear(orig_data, orig_width, orig_height, src_px, src_py)
                {
                    let dst_idx = ((ty * new_width + tx) * 4) as usize;
                    new_data[dst_idx] = color[0];
                    new_data[dst_idx + 1] = color[1];
                    new_data[dst_idx + 2] = color[2];
                    new_data[dst_idx + 3] = color[3];
                }
            }
        }

        self.sprites.display_data[idx] = new_data;
        self.sprites.display_widths[idx] = new_width;
        self.sprites.display_heights[idx] = new_height;
    }

    /// 应用旋转+缩放组合变换
    ///
    /// 在原始数据上同时应用旋转和缩放，结果覆盖显示数据。
    pub fn apply_sprite_transform(&mut self, id: u32, angle: f32, sx: f32, sy: f32) {
        let idx = id as usize;
        if !self.sprites.is_active(id) || sx.abs() < 0.001 || sy.abs() < 0.001 {
            return;
        }

        let orig_width = self.sprites.original_widths[idx];
        let orig_height = self.sprites.original_heights[idx];
        let orig_data = &self.sprites.original_data[idx];

        // 计算缩放后的尺寸
        let scaled_w = orig_width as f32 * sx.abs();
        let scaled_h = orig_height as f32 * sy.abs();

        // 计算旋转后的边界框
        let cos_a = angle.cos().abs();
        let sin_a = angle.sin().abs();
        let new_width = (scaled_w * cos_a + scaled_h * sin_a).ceil() as u32;
        let new_height = (scaled_w * sin_a + scaled_h * cos_a).ceil() as u32;

        if new_width == 0 || new_height == 0 {
            return;
        }

        let new_size = (new_width * new_height * 4) as usize;
        let mut new_data = vec![0u8; new_size];

        // 组合变换矩阵：先缩放后旋转
        let scale = Matrix3x3::scale(sx, sy);
        let rotation = Matrix3x3::rotation(-angle);
        let transform = rotation.multiply(&scale);
        let inverse = transform.inverse().unwrap_or_else(Matrix3x3::identity);

        let orig_half_w = orig_width as f32 / 2.0;
        let orig_half_h = orig_height as f32 / 2.0;
        let new_half_w = new_width as f32 / 2.0;
        let new_half_h = new_height as f32 / 2.0;

        // 逐像素变换
        for ty in 0..new_height {
            for tx in 0..new_width {
                let dst_x = tx as f32 - new_half_w;
                let dst_y = ty as f32 - new_half_h;

                let (src_x, src_y) = inverse.transform_point(dst_x, dst_y);
                let src_px = src_x + orig_half_w;
                let src_py = src_y + orig_half_h;

                if let Some(color) =
                    sample_bilinear(orig_data, orig_width, orig_height, src_px, src_py)
                {
                    let dst_idx = ((ty * new_width + tx) * 4) as usize;
                    new_data[dst_idx] = color[0];
                    new_data[dst_idx + 1] = color[1];
                    new_data[dst_idx + 2] = color[2];
                    new_data[dst_idx + 3] = color[3];
                }
            }
        }

        self.sprites.display_data[idx] = new_data;
        self.sprites.display_widths[idx] = new_width;
        self.sprites.display_heights[idx] = new_height;
    }

    /// 重置精灵图变换 (恢复到原始状态)
    pub fn reset_sprite_transform(&mut self, id: u32) {
        let idx = id as usize;
        if !self.sprites.is_active(id) {
            return;
        }

        self.sprites.display_data[idx] = self.sprites.original_data[idx].clone();
        self.sprites.display_widths[idx] = self.sprites.original_widths[idx];
        self.sprites.display_heights[idx] = self.sprites.original_heights[idx];
    }

    // ========== 场景操作 ==========

    /// 创建新场景
    pub fn create_scene(&mut self, width: u32, height: u32) -> u32 {
        self.scenes.add(width, height)
    }

    /// 设置场景 z-index
    pub fn set_scene_zindex(&mut self, id: u32, zindex: i32) {
        let idx = id as usize;
        if self.scenes.is_active(id) {
            self.scenes.zindexes[idx] = zindex;
        }
    }

    /// 获取场景 z-index
    pub fn get_scene_zindex(&self, id: u32) -> i32 {
        let idx = id as usize;
        if self.scenes.is_active(id) {
            self.scenes.zindexes[idx]
        } else {
            0
        }
    }

    /// 添加精灵图到场景
    pub fn add_to_scene(&mut self, sprite_id: u32) {
        self.add_sprite_to_scene(sprite_id, self.default_scene);
    }

    /// 添加精灵图到指定场景
    pub fn add_sprite_to_scene(&mut self, sprite_id: u32, scene_id: u32) {
        let scene_idx = scene_id as usize;
        if self.scenes.is_active(scene_id) && self.sprites.is_active(sprite_id) {
            if !self.scenes.sprite_ids[scene_idx].contains(&sprite_id) {
                self.scenes.sprite_ids[scene_idx].push(sprite_id);
            }
        }
    }

    /// 从场景移除精灵图
    pub fn remove_from_scene(&mut self, sprite_id: u32) {
        let scene_idx = self.default_scene as usize;
        if scene_idx < self.scenes.sprite_ids.len() {
            self.scenes.sprite_ids[scene_idx].retain(|&id| id != sprite_id);
        }
    }

    /// 设置场景背景色
    pub fn set_background_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        let idx = self.default_scene as usize;
        if idx < self.scenes.background_colors.len() {
            self.scenes.background_colors[idx] = [r, g, b, a];
        }
    }

    /// 设置采样方法
    pub fn set_sampling_method(&mut self, method: u8) {
        let idx = self.default_scene as usize;
        if idx < self.scenes.sampling_methods.len() {
            self.scenes.sampling_methods[idx] = SamplingMethod::from_u8(method);
        }
    }

    /// 获取当前采样方法
    pub fn get_sampling_method(&self) -> u8 {
        let idx = self.default_scene as usize;
        if idx < self.scenes.sampling_methods.len() {
            self.scenes.sampling_methods[idx].to_u8()
        } else {
            0
        }
    }

    /// 渲染一帧
    pub fn render(&mut self) {
        let scene_idx = self.default_scene as usize;
        if scene_idx >= self.scenes.data.len() {
            return;
        }

        let width = self.scenes.widths[scene_idx];
        let height = self.scenes.heights[scene_idx];
        let bg_color = self.scenes.background_colors[scene_idx];
        let sampling_method = self.scenes.sampling_methods[scene_idx];

        // 清空场景 (填充背景色)
        let scene_data = &mut self.scenes.data[scene_idx];
        for chunk in scene_data.chunks_exact_mut(4) {
            chunk[0] = bg_color[0];
            chunk[1] = bg_color[1];
            chunk[2] = bg_color[2];
            chunk[3] = bg_color[3];
        }

        // 收集并排序精灵图
        let mut sprite_ids: Vec<u32> = self.scenes.sprite_ids[scene_idx]
            .iter()
            .filter(|&&id| self.sprites.is_active(id))
            .cloned()
            .collect();
        sprite_ids.sort_by_key(|&id| self.sprites.zindexes[id as usize]);

        // 渲染每个精灵图
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;

        for sprite_id in sprite_ids {
            let idx = sprite_id as usize;
            let sprite_data = &self.sprites.display_data[idx];
            let sprite_w = self.sprites.display_widths[idx];
            let sprite_h = self.sprites.display_heights[idx];
            let pos_x = self.sprites.positions_x[idx];
            let pos_y = self.sprites.positions_y[idx];

            let half_w = sprite_w as f32 / 2.0;
            let half_h = sprite_h as f32 / 2.0;

            // 计算精灵图在场景中的边界
            let start_x = ((pos_x - half_w + center_x).floor() as i32).max(0) as u32;
            let end_x = ((pos_x + half_w + center_x).ceil() as i32).min(width as i32) as u32;
            let start_y = ((pos_y - half_h + center_y).floor() as i32).max(0) as u32;
            let end_y = ((pos_y + half_h + center_y).ceil() as i32).min(height as i32) as u32;

            // 逐像素渲染
            for ty in start_y..end_y {
                for tx in start_x..end_x {
                    let local_x = tx as f32 - center_x - pos_x + half_w;
                    let local_y = ty as f32 - center_y - pos_y + half_h;

                    let color = match sampling_method {
                        SamplingMethod::Nearest => {
                            sample_nearest(sprite_data, sprite_w, sprite_h, local_x, local_y)
                        }
                        SamplingMethod::Bilinear => {
                            sample_bilinear(sprite_data, sprite_w, sprite_h, local_x, local_y)
                        }
                        SamplingMethod::Supersampling => {
                            sample_supersampling(sprite_data, sprite_w, sprite_h, local_x, local_y)
                        }
                    };

                    if let Some(color) = color {
                        let dst_idx = ((ty * width + tx) * 4) as usize;
                        let src_a = color[3] as f32 / 255.0;

                        if src_a > 0.0 {
                            let dst_a = scene_data[dst_idx + 3] as f32 / 255.0;
                            let out_a = src_a + dst_a * (1.0 - src_a);

                            if out_a > 0.0 {
                                for i in 0..3 {
                                    let src_c = color[i] as f32;
                                    let dst_c = scene_data[dst_idx + i] as f32;
                                    scene_data[dst_idx + i] =
                                        ((src_c * src_a + dst_c * dst_a * (1.0 - src_a)) / out_a)
                                            as u8;
                                }
                                scene_data[dst_idx + 3] = (out_a * 255.0) as u8;
                            }
                        }
                    }
                }
            }
        }
    }

    /// 获取场景数据指针
    pub fn scene_data_ptr(&self) -> *const u8 {
        let idx = self.default_scene as usize;
        if idx < self.scenes.data.len() {
            self.scenes.data[idx].as_ptr()
        } else {
            std::ptr::null()
        }
    }

    /// 获取场景数据长度
    pub fn scene_data_len(&self) -> usize {
        let idx = self.default_scene as usize;
        if idx < self.scenes.data.len() {
            self.scenes.data[idx].len()
        } else {
            0
        }
    }

    /// 获取场景宽度
    pub fn scene_width(&self) -> u32 {
        let idx = self.default_scene as usize;
        if idx < self.scenes.widths.len() {
            self.scenes.widths[idx]
        } else {
            0
        }
    }

    /// 获取场景高度
    pub fn scene_height(&self) -> u32 {
        let idx = self.default_scene as usize;
        if idx < self.scenes.heights.len() {
            self.scenes.heights[idx]
        } else {
            0
        }
    }

    /// 调整场景尺寸
    pub fn resize_scene(&mut self, width: u32, height: u32) {
        let idx = self.default_scene as usize;
        if idx < self.scenes.data.len() {
            self.scenes.widths[idx] = width;
            self.scenes.heights[idx] = height;
            let new_size = (width * height * 4) as usize;
            self.scenes.data[idx].resize(new_size, 0);
        }
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
        assert_eq!(id, 0); // 第一个精灵图ID为0

        let id2 = world.create_rect_sprite(10, 10, 0, 255, 0, 255);
        assert_eq!(id2, 1); // 第二个精灵图ID为1
    }

    #[test]
    fn test_sprite_position() {
        let mut world = World::new(100, 100);
        let id = world.create_rect_sprite(10, 10, 255, 0, 0, 255);

        world.set_sprite_position(id, 50.0, 30.0);
        let pos = world.get_sprite_position(id).unwrap();
        assert_eq!(pos, vec![50.0, 30.0]);
    }

    #[test]
    fn test_sprite_zindex() {
        let mut world = World::new(100, 100);
        let id = world.create_rect_sprite(10, 10, 255, 0, 0, 255);

        world.set_sprite_zindex(id, 5);
        assert_eq!(world.get_sprite_zindex(id), 5);
    }

    #[test]
    fn test_sprite_removal() {
        let mut world = World::new(100, 100);
        let id = world.create_rect_sprite(10, 10, 255, 0, 0, 255);

        assert!(world.sprites.is_active(id));
        world.remove_sprite(id);
        assert!(!world.sprites.is_active(id));
    }

    #[test]
    fn test_render() {
        let mut world = World::new(100, 100);
        let id = world.create_rect_sprite(10, 10, 255, 0, 0, 255);
        world.add_to_scene(id);
        world.render();
        assert!(world.scene_data_len() > 0);
    }
}
