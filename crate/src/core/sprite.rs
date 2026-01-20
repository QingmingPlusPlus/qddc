//! 精灵图模块
//!
//! 精灵图是可变换的图像单元，包含像素数据和变换属性。

use crate::math::Matrix3x3;
use super::sampling::{SamplingMethod, sample_nearest, sample_bilinear, sample_supersampling};

/// 精灵图 - 可变换的图像单元
///
/// 精灵图以几何中心作为位置原点，支持平移、旋转、缩放变换。
#[derive(Debug, Clone)]
pub struct Sprite {
    /// 精灵图唯一标识符
    id: u32,
    /// RGBA 像素数据
    data: Vec<u8>,
    /// 原始宽度
    width: u32,
    /// 原始高度
    height: u32,
    /// 几何中心位置
    position: (f32, f32),
    /// 堆叠层级 (类似 CSS z-index)
    zindex: i32,
    /// 旋转角度 (弧度)
    rotation: f32,
    /// 缩放因子
    scale: (f32, f32),
}

impl Sprite {
    /// 创建新的精灵图
    ///
    /// # Arguments
    /// * `id` - 唯一标识符
    /// * `data` - RGBA 像素数据 (长度必须是 width * height * 4)
    /// * `width` - 图像宽度
    /// * `height` - 图像高度
    ///
    /// # Panics
    /// 如果 data 长度不匹配 width * height * 4
    pub fn new(id: u32, data: Vec<u8>, width: u32, height: u32) -> Self {
        let expected_len = (width * height * 4) as usize;
        assert_eq!(
            data.len(),
            expected_len,
            "Data length {} doesn't match expected {} ({}x{}x4)",
            data.len(),
            expected_len,
            width,
            height
        );

        Self {
            id,
            data,
            width,
            height,
            position: (0.0, 0.0),
            zindex: 0,
            rotation: 0.0,
            scale: (1.0, 1.0),
        }
    }

    /// 获取精灵图 ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// 获取几何中心位置
    pub fn position(&self) -> (f32, f32) {
        self.position
    }

    /// 获取尺寸 (当前变换后的逻辑尺寸)
    pub fn size(&self) -> (u32, u32) {
        (
            (self.width as f32 * self.scale.0.abs()) as u32,
            (self.height as f32 * self.scale.1.abs()) as u32,
        )
    }

    /// 获取原始尺寸
    pub fn original_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// 获取 z-index
    pub fn zindex(&self) -> i32 {
        self.zindex
    }

    /// 设置 z-index
    pub fn set_zindex(&mut self, zindex: i32) {
        self.zindex = zindex;
    }

    /// 获取旋转角度 (弧度)
    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    /// 获取缩放因子
    pub fn scale(&self) -> (f32, f32) {
        self.scale
    }

    /// 获取像素数据
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// 平移精灵图
    ///
    /// # Arguments
    /// * `dx` - X 方向平移量
    /// * `dy` - Y 方向平移量
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.position.0 += dx;
        self.position.1 += dy;
    }

    /// 设置精灵图位置
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    /// 旋转精灵图
    ///
    /// 这会累加到当前旋转角度，并重新生成像素数据。
    ///
    /// # Arguments
    /// * `angle` - 旋转角度 (弧度，逆时针为正)
    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
        // 归一化到 [0, 2π)
        while self.rotation < 0.0 {
            self.rotation += std::f32::consts::TAU;
        }
        while self.rotation >= std::f32::consts::TAU {
            self.rotation -= std::f32::consts::TAU;
        }
    }

    /// 设置旋转角度
    pub fn set_rotation(&mut self, angle: f32) {
        self.rotation = angle;
    }

    /// 缩放精灵图
    ///
    /// 这会累乘到当前缩放因子。
    ///
    /// # Arguments
    /// * `sx` - X 方向缩放因子
    /// * `sy` - Y 方向缩放因子
    pub fn scale_by(&mut self, sx: f32, sy: f32) {
        self.scale.0 *= sx;
        self.scale.1 *= sy;
    }

    /// 设置缩放因子
    pub fn set_scale(&mut self, sx: f32, sy: f32) {
        self.scale = (sx, sy);
    }

    /// 获取当前变换矩阵
    ///
    /// 变换顺序：缩放 -> 旋转 -> 平移
    pub fn transform_matrix(&self) -> Matrix3x3 {
        let scale = Matrix3x3::scale(self.scale.0, self.scale.1);
        let rotation = Matrix3x3::rotation(self.rotation);
        let translation = Matrix3x3::translation(self.position.0, self.position.1);

        // T * R * S
        translation.multiply(&rotation).multiply(&scale)
    }

    /// 在目标缓冲区中渲染此精灵图
    ///
    /// # Arguments
    /// * `target` - 目标 RGBA 像素缓冲区
    /// * `target_width` - 目标缓冲区宽度
    /// * `target_height` - 目标缓冲区高度
    /// * `sampling_method` - 采样方法
    pub fn render_to(
        &self,
        target: &mut [u8],
        target_width: u32,
        target_height: u32,
        sampling_method: SamplingMethod,
    ) {
        let transform = self.transform_matrix();
        let inverse = match transform.inverse() {
            Some(inv) => inv,
            None => return, // 变换不可逆，跳过渲染
        };

        let half_w = self.width as f32 / 2.0;
        let half_h = self.height as f32 / 2.0;

        // 计算变换后的包围盒
        let corners = [
            (-half_w, -half_h),
            (half_w, -half_h),
            (half_w, half_h),
            (-half_w, half_h),
        ];

        let transformed_corners: Vec<(f32, f32)> = corners
            .iter()
            .map(|&(x, y)| transform.transform_point(x, y))
            .collect();

        let min_x = transformed_corners
            .iter()
            .map(|c| c.0)
            .fold(f32::INFINITY, f32::min);
        let max_x = transformed_corners
            .iter()
            .map(|c| c.0)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = transformed_corners
            .iter()
            .map(|c| c.1)
            .fold(f32::INFINITY, f32::min);
        let max_y = transformed_corners
            .iter()
            .map(|c| c.1)
            .fold(f32::NEG_INFINITY, f32::max);

        // 场景坐标系：中心在 (target_width/2, target_height/2)
        let center_x = target_width as f32 / 2.0;
        let center_y = target_height as f32 / 2.0;

        // 裁剪到目标区域
        let start_x = ((min_x + center_x).floor() as i32).max(0) as u32;
        let end_x = ((max_x + center_x).ceil() as i32).min(target_width as i32) as u32;
        let start_y = ((min_y + center_y).floor() as i32).max(0) as u32;
        let end_y = ((max_y + center_y).ceil() as i32).min(target_height as i32) as u32;

        // 逐像素渲染
        for ty in start_y..end_y {
            for tx in start_x..end_x {
                // 转换到精灵图局部坐标
                let world_x = tx as f32 - center_x;
                let world_y = ty as f32 - center_y;
                let (local_x, local_y) = inverse.transform_point(world_x, world_y);

                // 转换到像素坐标 (原点在左上角)
                let px = local_x + half_w;
                let py = local_y + half_h;

                // 根据采样方法获取颜色
                let sampled_color = match sampling_method {
                    SamplingMethod::Nearest => {
                        sample_nearest(&self.data, self.width, self.height, px, py)
                    }
                    SamplingMethod::Bilinear => {
                        sample_bilinear(&self.data, self.width, self.height, px, py)
                    }
                    SamplingMethod::Supersampling => {
                        sample_supersampling(&self.data, self.width, self.height, px, py)
                    }
                };

                if let Some(color) = sampled_color {
                    let dst_idx = ((ty * target_width + tx) * 4) as usize;

                    // Alpha 混合
                    let src_a = color[3] as f32 / 255.0;
                    if src_a > 0.0 {
                        let dst_a = target[dst_idx + 3] as f32 / 255.0;
                        let out_a = src_a + dst_a * (1.0 - src_a);

                        if out_a > 0.0 {
                            for i in 0..3 {
                                let src_c = color[i] as f32;
                                let dst_c = target[dst_idx + i] as f32;
                                target[dst_idx + i] =
                                    ((src_c * src_a + dst_c * dst_a * (1.0 - src_a)) / out_a)
                                        as u8;
                            }
                            target[dst_idx + 3] = (out_a * 255.0) as u8;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sprite(width: u32, height: u32, color: [u8; 4]) -> Sprite {
        let size = (width * height * 4) as usize;
        let mut data = vec![0u8; size];
        for i in 0..(width * height) as usize {
            data[i * 4] = color[0];
            data[i * 4 + 1] = color[1];
            data[i * 4 + 2] = color[2];
            data[i * 4 + 3] = color[3];
        }
        Sprite::new(1, data, width, height)
    }

    #[test]
    fn test_new_sprite() {
        let sprite = create_test_sprite(10, 10, [255, 0, 0, 255]);
        assert_eq!(sprite.id(), 1);
        assert_eq!(sprite.position(), (0.0, 0.0));
        assert_eq!(sprite.original_size(), (10, 10));
    }

    #[test]
    fn test_translate() {
        let mut sprite = create_test_sprite(10, 10, [255, 0, 0, 255]);
        sprite.translate(5.0, 10.0);
        assert_eq!(sprite.position(), (5.0, 10.0));
    }

    #[test]
    fn test_scale() {
        let mut sprite = create_test_sprite(10, 10, [255, 0, 0, 255]);
        sprite.set_scale(2.0, 2.0);
        assert_eq!(sprite.size(), (20, 20));
    }

    #[test]
    fn test_rotation() {
        let mut sprite = create_test_sprite(10, 10, [255, 0, 0, 255]);
        sprite.rotate(std::f32::consts::PI);
        assert!((sprite.rotation() - std::f32::consts::PI).abs() < 0.001);
    }
}
