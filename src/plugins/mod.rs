pub mod debug_plugin;
pub mod layers_plugin;
pub mod simulation_plugin;
#[cfg(feature = "v7_worldgen")]
pub mod worldgen_plugin;

pub use crate::runtime_platform::camera_controller_3d::Camera3dPlugin;
pub use crate::runtime_platform::click_to_move::ClickToMovePlugin;
pub use crate::runtime_platform::debug_observability::ObservabilityPlugin;
pub use crate::runtime_platform::input_capture::InputCapturePlugin;
pub use crate::runtime_platform::render_bridge_3d::RenderBridge3dPlugin;
pub use crate::runtime_platform::scenario_isolation::ScenarioIsolationPlugin;
pub use crate::runtime_platform::simulation_tick::SimulationTickPlugin;
pub use debug_plugin::DebugPlugin;
pub use layers_plugin::LayersPlugin;
pub use simulation_plugin::SimulationPlugin;
#[cfg(feature = "v7_worldgen")]
pub use worldgen_plugin::WorldgenPlugin;
