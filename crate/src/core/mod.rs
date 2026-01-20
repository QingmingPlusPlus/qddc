//! ECS 核心模块
//!
//! 提供简易的 ECS 架构，包含精灵图和场景两种实体类型。

mod sampling;
mod sprite;
mod scene;
mod world;

pub use sampling::SamplingMethod;
pub use sprite::Sprite;
pub use scene::Scene;
pub use world::World;
