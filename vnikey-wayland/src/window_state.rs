use std::collections::HashMap;

pub struct WindowStateManager {
    states: HashMap<String, bool>,
    pub current_active_window: Option<String>,
}

impl WindowStateManager {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            current_active_window: None,
        }
    }

    pub fn set_active_window(&mut self, window_id: String) {
        self.current_active_window = Some(window_id);
    }

    pub fn get_state_for_current_window(&self) -> Option<bool> {
        if let Some(window_id) = &self.current_active_window {
            self.states.get(window_id).copied()
        } else {
            None
        }
    }

    pub fn save_state_for_current_window(&mut self, state: bool) {
        if let Some(window_id) = &self.current_active_window {
            self.states.insert(window_id.clone(), state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_window_switching() {
        let mut manager = WindowStateManager::new();

        // App A
        manager.set_active_window("AppA".to_string());
        manager.save_state_for_current_window(true);
        assert_eq!(manager.get_state_for_current_window(), Some(true));

        // Switch to App B
        manager.set_active_window("AppB".to_string());
        assert_eq!(manager.get_state_for_current_window(), None); // Default is None
        manager.save_state_for_current_window(false);
        assert_eq!(manager.get_state_for_current_window(), Some(false));

        // Switch back to App A
        manager.set_active_window("AppA".to_string());
        assert_eq!(manager.get_state_for_current_window(), Some(true));
    }

    #[test]
    fn test_empty_or_null_id() {
        let mut manager = WindowStateManager::new();

        // Should not panic, and getting state should just return None
        assert_eq!(manager.get_state_for_current_window(), None);

        // Saving state with no active window should not crash
        manager.save_state_for_current_window(true);

        // Still None
        assert_eq!(manager.get_state_for_current_window(), None);

        // Empty string
        manager.set_active_window("".to_string());
        assert_eq!(manager.get_state_for_current_window(), None);
        manager.save_state_for_current_window(true);
        assert_eq!(manager.get_state_for_current_window(), Some(true));
    }

    #[test]
    fn test_toggle_rapid_typing() {
        let mut manager = WindowStateManager::new();

        manager.set_active_window("AppRapid".to_string());
        manager.save_state_for_current_window(false);

        // Rapid toggle
        for i in 0..100 {
            let state = i % 2 == 0;
            manager.save_state_for_current_window(state);
            assert_eq!(manager.get_state_for_current_window(), Some(state));
        }
    }
}
