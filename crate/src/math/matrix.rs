//! 矩阵计算库
//!
//! 实现 2D 变换所需的 3x3 矩阵运算。

/// 3x3 变换矩阵 (用于 2D 仿射变换)
///
/// 矩阵采用行优先存储:
/// ```text
/// | m[0] m[1] m[2] |   | a  b  tx |
/// | m[3] m[4] m[5] | = | c  d  ty |
/// | m[6] m[7] m[8] |   | 0  0  1  |
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Matrix3x3 {
    data: [f32; 9],
}

impl Matrix3x3 {
    /// 创建单位矩阵
    pub fn identity() -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 0.0, 1.0,
            ],
        }
    }

    /// 创建平移矩阵
    ///
    /// # Arguments
    /// * `tx` - X 方向平移量
    /// * `ty` - Y 方向平移量
    pub fn translation(tx: f32, ty: f32) -> Self {
        Self {
            data: [
                1.0, 0.0, tx,
                0.0, 1.0, ty,
                0.0, 0.0, 1.0,
            ],
        }
    }

    /// 创建旋转矩阵 (绕原点旋转)
    ///
    /// # Arguments
    /// * `angle` - 旋转角度 (弧度，逆时针为正)
    pub fn rotation(angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            data: [
                cos_a, -sin_a, 0.0,
                sin_a,  cos_a, 0.0,
                0.0,    0.0,   1.0,
            ],
        }
    }

    /// 创建缩放矩阵 (以原点为中心)
    ///
    /// # Arguments
    /// * `sx` - X 方向缩放因子
    /// * `sy` - Y 方向缩放因子
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            data: [
                sx,  0.0, 0.0,
                0.0, sy,  0.0,
                0.0, 0.0, 1.0,
            ],
        }
    }

    /// 矩阵乘法: self * other
    ///
    /// 注意：变换顺序是从右到左应用的
    pub fn multiply(&self, other: &Self) -> Self {
        let a = &self.data;
        let b = &other.data;
        
        Self {
            data: [
                a[0]*b[0] + a[1]*b[3] + a[2]*b[6],
                a[0]*b[1] + a[1]*b[4] + a[2]*b[7],
                a[0]*b[2] + a[1]*b[5] + a[2]*b[8],
                
                a[3]*b[0] + a[4]*b[3] + a[5]*b[6],
                a[3]*b[1] + a[4]*b[4] + a[5]*b[7],
                a[3]*b[2] + a[4]*b[5] + a[5]*b[8],
                
                a[6]*b[0] + a[7]*b[3] + a[8]*b[6],
                a[6]*b[1] + a[7]*b[4] + a[8]*b[7],
                a[6]*b[2] + a[7]*b[5] + a[8]*b[8],
            ],
        }
    }

    /// 变换一个点
    ///
    /// # Arguments
    /// * `x` - 点的 X 坐标
    /// * `y` - 点的 Y 坐标
    ///
    /// # Returns
    /// 变换后的 (x', y') 坐标
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        let m = &self.data;
        let new_x = m[0] * x + m[1] * y + m[2];
        let new_y = m[3] * x + m[4] * y + m[5];
        (new_x, new_y)
    }

    /// 计算逆矩阵
    ///
    /// # Returns
    /// 逆矩阵，如果矩阵不可逆则返回 None
    pub fn inverse(&self) -> Option<Self> {
        let m = &self.data;
        
        // 计算行列式
        let det = m[0] * (m[4] * m[8] - m[5] * m[7])
                - m[1] * (m[3] * m[8] - m[5] * m[6])
                + m[2] * (m[3] * m[7] - m[4] * m[6]);
        
        if det.abs() < 1e-10 {
            return None;
        }
        
        let inv_det = 1.0 / det;
        
        Some(Self {
            data: [
                (m[4] * m[8] - m[5] * m[7]) * inv_det,
                (m[2] * m[7] - m[1] * m[8]) * inv_det,
                (m[1] * m[5] - m[2] * m[4]) * inv_det,
                
                (m[5] * m[6] - m[3] * m[8]) * inv_det,
                (m[0] * m[8] - m[2] * m[6]) * inv_det,
                (m[2] * m[3] - m[0] * m[5]) * inv_det,
                
                (m[3] * m[7] - m[4] * m[6]) * inv_det,
                (m[1] * m[6] - m[0] * m[7]) * inv_det,
                (m[0] * m[4] - m[1] * m[3]) * inv_det,
            ],
        })
    }

    /// 获取矩阵数据的只读引用
    pub fn data(&self) -> &[f32; 9] {
        &self.data
    }
}

impl Default for Matrix3x3 {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_identity() {
        let m = Matrix3x3::identity();
        let (x, y) = m.transform_point(3.0, 4.0);
        assert!(approx_eq(x, 3.0));
        assert!(approx_eq(y, 4.0));
    }

    #[test]
    fn test_translation() {
        let m = Matrix3x3::translation(10.0, 20.0);
        let (x, y) = m.transform_point(5.0, 5.0);
        assert!(approx_eq(x, 15.0));
        assert!(approx_eq(y, 25.0));
    }

    #[test]
    fn test_rotation_90_degrees() {
        let m = Matrix3x3::rotation(PI / 2.0);
        let (x, y) = m.transform_point(1.0, 0.0);
        assert!(approx_eq(x, 0.0));
        assert!(approx_eq(y, 1.0));
    }

    #[test]
    fn test_scale() {
        let m = Matrix3x3::scale(2.0, 3.0);
        let (x, y) = m.transform_point(5.0, 5.0);
        assert!(approx_eq(x, 10.0));
        assert!(approx_eq(y, 15.0));
    }

    #[test]
    fn test_multiply() {
        // 先缩放再平移
        let scale = Matrix3x3::scale(2.0, 2.0);
        let translate = Matrix3x3::translation(10.0, 10.0);
        let combined = translate.multiply(&scale);
        
        let (x, y) = combined.transform_point(5.0, 5.0);
        // 5 * 2 = 10, 10 + 10 = 20
        assert!(approx_eq(x, 20.0));
        assert!(approx_eq(y, 20.0));
    }

    #[test]
    fn test_inverse() {
        let m = Matrix3x3::translation(10.0, 20.0);
        let inv = m.inverse().unwrap();
        let identity = m.multiply(&inv);
        
        let (x, y) = identity.transform_point(5.0, 5.0);
        assert!(approx_eq(x, 5.0));
        assert!(approx_eq(y, 5.0));
    }
}
