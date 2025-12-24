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
