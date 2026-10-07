//! Robotics producers built on the shared Cerul engine.
pub use cerul::{
    annotations, config, diagnostics, episode, events, index, media, providers, search, status,
    storage,
};
pub mod annotate;
pub mod lerobot;

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod core_discovery_tests;
#[cfg(test)]
mod core_pipeline_tests;
