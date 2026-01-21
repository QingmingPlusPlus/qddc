//! ECS 核心模块
//!
//! 提供纯数据导向的 ECS 架构，使用数组存储精灵图和场景数据。

mod sampling;
mod world;

pub use sampling::SamplingMethod;
pub use world::World;
