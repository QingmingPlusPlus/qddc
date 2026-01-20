//! 采样方法模块
//!
//! 提供不同的像素采样算法用于精灵图渲染。

/// 采样方法枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SamplingMethod {
    /// 最近邻采样 (速度快，有锯齿)
    #[default]
    Nearest,
    /// 双线性插值 (平滑，轻微模糊)
    Bilinear,
    /// 超采样抗锯齿 (质量最好，性能开销大)
    Supersampling,
}

impl SamplingMethod {
    /// 从 u8 值创建采样方法
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => SamplingMethod::Nearest,
            1 => SamplingMethod::Bilinear,
            2 => SamplingMethod::Supersampling,
            _ => SamplingMethod::Nearest,
        }
    }

    /// 转换为 u8 值
    pub fn to_u8(self) -> u8 {
        match self {
            SamplingMethod::Nearest => 0,
            SamplingMethod::Bilinear => 1,
            SamplingMethod::Supersampling => 2,
        }
    }
}

/// 最近邻采样
///
/// 直接取最近的整数像素坐标对应的颜色。
/// 速度最快，但会产生锯齿。
///
/// # Arguments
/// * `data` - 源像素数据 (RGBA)
/// * `width` - 源图像宽度
/// * `height` - 源图像高度
/// * `px` - 采样 X 坐标 (像素坐标系，原点在左上角)
/// * `py` - 采样 Y 坐标
///
/// # Returns
/// RGBA 颜色值，如果坐标越界则返回 None
pub fn sample_nearest(
    data: &[u8],
    width: u32,
    height: u32,
    px: f32,
    py: f32,
) -> Option<[u8; 4]> {
    let src_x = px.round() as i32;
    let src_y = py.round() as i32;

    if src_x >= 0 && src_x < width as i32 && src_y >= 0 && src_y < height as i32 {
        let idx = ((src_y as u32 * width + src_x as u32) * 4) as usize;
        Some([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]])
    } else {
        None
    }
}

/// 双线性插值采样
///
/// 对邻近的 4 个像素进行加权平均，产生平滑的结果。
/// 速度适中，边缘更平滑。
///
/// # Arguments
/// * `data` - 源像素数据 (RGBA)
/// * `width` - 源图像宽度
/// * `height` - 源图像高度
/// * `px` - 采样 X 坐标 (像素坐标系，原点在左上角)
/// * `py` - 采样 Y 坐标
///
/// # Returns
/// RGBA 颜色值，如果坐标完全越界则返回 None
pub fn sample_bilinear(
    data: &[u8],
    width: u32,
    height: u32,
    px: f32,
    py: f32,
) -> Option<[u8; 4]> {
    // 坐标调整：采样点在像素中心
    let px = px - 0.5;
    let py = py - 0.5;

    let x0 = px.floor() as i32;
    let y0 = py.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    // 小数部分作为插值权重
    let fx = px - px.floor();
    let fy = py - py.floor();

    // 获取四个角的像素颜色
    let get_pixel = |x: i32, y: i32| -> [f32; 4] {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            let idx = ((y as u32 * width + x as u32) * 4) as usize;
            [
                data[idx] as f32,
                data[idx + 1] as f32,
                data[idx + 2] as f32,
                data[idx + 3] as f32,
            ]
        } else {
            [0.0, 0.0, 0.0, 0.0] // 越界返回透明
        }
    };

    let c00 = get_pixel(x0, y0);
    let c10 = get_pixel(x1, y0);
    let c01 = get_pixel(x0, y1);
    let c11 = get_pixel(x1, y1);

    // 检查是否完全越界
    let all_transparent = c00[3] == 0.0 && c10[3] == 0.0 && c01[3] == 0.0 && c11[3] == 0.0;
    if all_transparent {
        // 如果是边缘，检查最近邻是否有效
        let nx = px.round() as i32;
        let ny = py.round() as i32;
        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
            let idx = ((ny as u32 * width + nx as u32) * 4) as usize;
            return Some([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]);
        }
        return None;
    }

    // 双线性插值
    // color = (1-fx)(1-fy)*c00 + fx*(1-fy)*c10 + (1-fx)*fy*c01 + fx*fy*c11
    let w00 = (1.0 - fx) * (1.0 - fy);
    let w10 = fx * (1.0 - fy);
    let w01 = (1.0 - fx) * fy;
    let w11 = fx * fy;

    let mut result = [0u8; 4];
    for i in 0..4 {
        let value = w00 * c00[i] + w10 * c10[i] + w01 * c01[i] + w11 * c11[i];
        result[i] = value.clamp(0.0, 255.0) as u8;
    }

    Some(result)
}

/// 超采样抗锯齿 (2x2)
///
/// 对每个目标像素采样 4 个子像素点，取平均值。
/// 质量最好，但性能开销较大。
///
/// # Arguments
/// * `data` - 源像素数据 (RGBA)
/// * `width` - 源图像宽度  
/// * `height` - 源图像高度
/// * `px` - 采样 X 坐标 (像素坐标系，原点在左上角)
/// * `py` - 采样 Y 坐标
/// * `inverse` - 逆变换矩阵，用于将目标像素映射回源像素
/// * `half_w` - 源图像半宽
/// * `half_h` - 源图像半高
///
/// # Returns
/// RGBA 颜色值
pub fn sample_supersampling(
    data: &[u8],
    width: u32,
    height: u32,
    px: f32,
    py: f32,
) -> Option<[u8; 4]> {
    // 2x2 超采样：在像素内采样 4 个点
    let offsets = [
        (-0.25, -0.25),
        (0.25, -0.25),
        (-0.25, 0.25),
        (0.25, 0.25),
    ];

    let mut r_sum = 0.0f32;
    let mut g_sum = 0.0f32;
    let mut b_sum = 0.0f32;
    let mut a_sum = 0.0f32;
    let mut sample_count = 0;

    for (ox, oy) in offsets {
        let sample_x = px + ox;
        let sample_y = py + oy;

        if let Some(color) = sample_nearest(data, width, height, sample_x, sample_y) {
            r_sum += color[0] as f32;
            g_sum += color[1] as f32;
            b_sum += color[2] as f32;
            a_sum += color[3] as f32;
            sample_count += 1;
        }
    }

    if sample_count == 0 {
        return None;
    }

    let count = sample_count as f32;
    Some([
        (r_sum / count) as u8,
        (g_sum / count) as u8,
        (b_sum / count) as u8,
        (a_sum / count) as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image() -> (Vec<u8>, u32, u32) {
        // 创建 2x2 测试图像
        // [红, 绿]
        // [蓝, 白]
        let data = vec![
            255, 0, 0, 255,     // 红 (0,0)
            0, 255, 0, 255,     // 绿 (1,0)
            0, 0, 255, 255,     // 蓝 (0,1)
            255, 255, 255, 255, // 白 (1,1)
        ];
        (data, 2, 2)
    }

    #[test]
    fn test_sampling_method_conversion() {
        assert_eq!(SamplingMethod::from_u8(0), SamplingMethod::Nearest);
        assert_eq!(SamplingMethod::from_u8(1), SamplingMethod::Bilinear);
        assert_eq!(SamplingMethod::from_u8(2), SamplingMethod::Supersampling);
        assert_eq!(SamplingMethod::from_u8(99), SamplingMethod::Nearest);

        assert_eq!(SamplingMethod::Nearest.to_u8(), 0);
        assert_eq!(SamplingMethod::Bilinear.to_u8(), 1);
        assert_eq!(SamplingMethod::Supersampling.to_u8(), 2);
    }

    #[test]
    fn test_sample_nearest() {
        let (data, width, height) = create_test_image();

        // 采样红色像素
        let color = sample_nearest(&data, width, height, 0.0, 0.0).unwrap();
        assert_eq!(color, [255, 0, 0, 255]);

        // 采样绿色像素
        let color = sample_nearest(&data, width, height, 1.0, 0.0).unwrap();
        assert_eq!(color, [0, 255, 0, 255]);

        // 越界返回 None
        assert!(sample_nearest(&data, width, height, -1.0, 0.0).is_none());
    }

    #[test]
    fn test_sample_bilinear() {
        let (data, width, height) = create_test_image();

        // 中心点应该是四色混合
        let color = sample_bilinear(&data, width, height, 1.0, 1.0).unwrap();
        // 混合后应该接近灰色
        assert!(color[0] > 100 && color[0] < 200);
    }

    #[test]
    fn test_sample_supersampling() {
        let (data, width, height) = create_test_image();

        // 角落应该返回该像素颜色
        let color = sample_supersampling(&data, width, height, 0.0, 0.0).unwrap();
        assert_eq!(color, [255, 0, 0, 255]);
    }
}
