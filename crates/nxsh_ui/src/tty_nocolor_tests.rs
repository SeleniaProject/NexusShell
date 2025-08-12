#[cfg(test)]
mod tty_nocolor_tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;
    use once_cell::sync::Lazy;

    // Global lock to serialize environment-variable touching tests
    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn test_tty_supports_color_without_env() {
        let _g = ENV_LOCK.lock().unwrap();
        // Clear environment variables
        env::remove_var("NXSH_TTY_NOCOLOR");
        env::remove_var("NO_COLOR");
        
        // Should support colors by default
        assert!(crate::tui::supports_color());
    }

    #[test] 
    fn test_tty_nocolor_env_disables_colors() {
        let _g = ENV_LOCK.lock().unwrap();
        // Set NXSH_TTY_NOCOLOR environment variable
        env::set_var("NXSH_TTY_NOCOLOR", "1");
        
        // Should disable colors
        assert!(!crate::tui::supports_color());
        
        // Clean up
        env::remove_var("NXSH_TTY_NOCOLOR");
    }

    #[test]
    fn test_no_color_env_disables_colors() {
        let _g = ENV_LOCK.lock().unwrap();
        // Set NO_COLOR environment variable
        env::set_var("NO_COLOR", "1");
        
        // Should disable colors
        assert!(!crate::tui::supports_color());
        
        // Clean up
        env::remove_var("NO_COLOR");
    }

    #[test]
    fn test_empty_no_color_enables_colors() {
        let _g = ENV_LOCK.lock().unwrap();
        // Set empty NO_COLOR environment variable
        env::set_var("NO_COLOR", "");
        
        // Should enable colors when NO_COLOR is empty
        assert!(crate::tui::supports_color());
        
        // Clean up
        env::remove_var("NO_COLOR");
    }

    #[tokio::test]
    async fn test_accessibility_manager_blind_mode() {
        let _g = ENV_LOCK.lock().unwrap();
        // Clear environment first
        env::remove_var("NXSH_TTY_NOCOLOR");
        env::remove_var("NO_COLOR");
        
        let mut manager = crate::accessibility::AccessibilityManager::new().unwrap();
        
        // Initially should not be in blind mode
        assert!(!manager.is_blind_mode_enabled().await);
        assert!(!manager.are_colors_disabled().await);
        
        // Enable blind mode
        manager.enable_blind_mode().await.unwrap();
        assert!(manager.is_blind_mode_enabled().await);
        assert!(manager.are_colors_disabled().await);
        
        // Disable blind mode
        manager.disable_blind_mode().await.unwrap();
        assert!(!manager.is_blind_mode_enabled().await);
    }

    #[tokio::test]
    async fn test_accessibility_manager_with_env_var() {
        let _g = ENV_LOCK.lock().unwrap();
        // Set NXSH_TTY_NOCOLOR environment variable
        env::set_var("NXSH_TTY_NOCOLOR", "1");
        
        let mut manager = crate::accessibility::AccessibilityManager::new().unwrap();
        manager.initialize().await.unwrap();
        
        // Should be in blind mode due to environment variable
        assert!(manager.is_blind_mode_enabled().await);
        assert!(manager.are_colors_disabled().await);
        
        // Clean up
        env::remove_var("NXSH_TTY_NOCOLOR");
    }
}
