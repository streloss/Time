pub mod player;

use player::PlayerManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct AudioState(pub Arc<PlayerManager>);

impl AudioState {
    pub fn new() -> Self {
        Self(Arc::new(PlayerManager::new()))
    }
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new()
    }
}
