//! Splash screen animation state

use std::time::{Duration, Instant};

/// Animation phase for splash screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplashPhase {
    /// Static logo display
    Display,
    /// Logo animating upward
    ScrollUp,
    /// Animation finished
    Complete,
}

/// Logo style options
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogoStyle {
    /// Large figlet-style block letters
    #[default]
    Block,
    /// Thin lines, minimalist
    Elegant,
    /// 8-bit pixelated style
    Retro,
}

/// Splash screen animation state
#[derive(Debug)]
pub struct SplashState {
    /// When the splash started
    pub start_time: Instant,
    /// Selected logo style
    #[allow(dead_code)]
    pub logo_style: LogoStyle,
    /// Current animation phase
    pub phase: SplashPhase,
    /// Current vertical offset (for scroll animation)
    pub scroll_offset: f32,
}

impl SplashState {
    /// Display duration before animation starts (1.3 seconds)
    const DISPLAY_DURATION: Duration = Duration::from_millis(1300);
    /// Duration of scroll-up animation (slower - 800ms)
    const ANIMATION_DURATION: Duration = Duration::from_millis(800);

    pub fn new(logo_style: LogoStyle) -> Self {
        Self {
            start_time: Instant::now(),
            logo_style,
            phase: SplashPhase::Display,
            scroll_offset: 0.0,
        }
    }

    /// Update animation state based on elapsed time
    pub fn update(&mut self, terminal_height: u16) {
        let elapsed = self.start_time.elapsed();

        if elapsed < Self::DISPLAY_DURATION {
            self.phase = SplashPhase::Display;
            self.scroll_offset = 0.0;
        } else if elapsed < Self::DISPLAY_DURATION + Self::ANIMATION_DURATION {
            self.phase = SplashPhase::ScrollUp;
            // Calculate progress (0.0 to 1.0)
            let animation_elapsed = elapsed - Self::DISPLAY_DURATION;
            let progress = animation_elapsed.as_secs_f32() / Self::ANIMATION_DURATION.as_secs_f32();
            // Apply easing (cubic ease-out for smooth deceleration)
            let eased = simple_easing::cubic_out(progress);
            // Calculate offset - scroll completely off the top of the screen
            // Move from center to fully off top (half height + logo height buffer)
            self.scroll_offset = eased * (terminal_height as f32);
        } else {
            self.phase = SplashPhase::Complete;
        }
    }

    /// Skip to completion (user pressed a key)
    pub fn skip(&mut self) {
        self.phase = SplashPhase::Complete;
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.phase == SplashPhase::Complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod logo_style {
        use super::*;

        #[test]
        fn test_default_is_block() {
            let style = LogoStyle::default();
            assert_eq!(style, LogoStyle::Block);
        }

        #[test]
        fn test_all_variants_exist() {
            // Just ensure all variants can be used
            let _block = LogoStyle::Block;
            let _elegant = LogoStyle::Elegant;
            let _retro = LogoStyle::Retro;
        }
    }

    mod splash_phase {
        use super::*;

        #[test]
        fn test_phases_are_distinct() {
            assert_ne!(SplashPhase::Display, SplashPhase::ScrollUp);
            assert_ne!(SplashPhase::Display, SplashPhase::Complete);
            assert_ne!(SplashPhase::ScrollUp, SplashPhase::Complete);
        }
    }

    mod splash_state {
        use super::*;

        #[test]
        fn test_new_starts_in_display_phase() {
            let state = SplashState::new(LogoStyle::Block);
            assert_eq!(state.phase, SplashPhase::Display);
            assert_eq!(state.scroll_offset, 0.0);
            assert_eq!(state.logo_style, LogoStyle::Block);
        }

        #[test]
        fn test_new_with_different_styles() {
            let elegant = SplashState::new(LogoStyle::Elegant);
            assert_eq!(elegant.logo_style, LogoStyle::Elegant);

            let retro = SplashState::new(LogoStyle::Retro);
            assert_eq!(retro.logo_style, LogoStyle::Retro);
        }

        #[test]
        fn test_skip_immediately_completes() {
            let mut state = SplashState::new(LogoStyle::Block);
            assert!(!state.is_complete());

            state.skip();

            assert!(state.is_complete());
            assert_eq!(state.phase, SplashPhase::Complete);
        }

        #[test]
        fn test_is_complete_returns_false_initially() {
            let state = SplashState::new(LogoStyle::Block);
            assert!(!state.is_complete());
        }

        #[test]
        fn test_is_complete_returns_true_after_skip() {
            let mut state = SplashState::new(LogoStyle::Block);
            state.skip();
            assert!(state.is_complete());
        }

        #[test]
        fn test_update_stays_in_display_phase_initially() {
            let mut state = SplashState::new(LogoStyle::Block);
            // Call update immediately after creation
            state.update(24);

            // Should still be in display phase
            assert_eq!(state.phase, SplashPhase::Display);
            assert_eq!(state.scroll_offset, 0.0);
        }

        #[test]
        fn test_multiple_skips_do_not_break() {
            let mut state = SplashState::new(LogoStyle::Block);
            state.skip();
            state.skip();
            state.skip();
            assert!(state.is_complete());
        }

        #[test]
        fn test_display_duration_constant() {
            // Verify constant is accessible (compile-time check)
            let duration = SplashState::DISPLAY_DURATION;
            assert!(duration.as_millis() > 0);
        }

        #[test]
        fn test_animation_duration_constant() {
            // Verify constant is accessible (compile-time check)
            let duration = SplashState::ANIMATION_DURATION;
            assert!(duration.as_millis() > 0);
        }

        #[test]
        fn test_scroll_offset_is_zero_in_display_phase() {
            let mut state = SplashState::new(LogoStyle::Block);
            state.update(40);
            assert_eq!(state.scroll_offset, 0.0);
        }

        #[test]
        fn test_update_with_different_terminal_heights() {
            let mut state1 = SplashState::new(LogoStyle::Block);
            let mut state2 = SplashState::new(LogoStyle::Block);

            state1.update(24);
            state2.update(100);

            // Both should be in display phase initially
            assert_eq!(state1.phase, SplashPhase::Display);
            assert_eq!(state2.phase, SplashPhase::Display);
        }

        // Note: Testing the time-based transition is challenging without
        // a way to mock time. The behavior can be verified by:
        // - Checking Display phase immediately after creation (done above)
        // - Checking Complete phase after skip() (done above)
        // - Manual/integration testing for the animated transitions
    }
}
