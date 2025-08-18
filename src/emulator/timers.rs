//! Timer implementation for the Chip-8 emulator.
//!
//! This module implements the delay timer and sound timer that decrement
//! at 60Hz when non-zero, providing timing functionality for games.

use std::time::{Duration, Instant};

/// Timer frequency in Hz (60 Hz standard for Chip-8).
pub const TIMER_FREQUENCY: u32 = 60;

/// Timer period in milliseconds (1000ms / 60Hz ≈ 16.67ms).
pub const TIMER_PERIOD_MS: u64 = 1000 / TIMER_FREQUENCY as u64;

/// Chip-8 timer system.
/// 
/// The Chip-8 has two 8-bit timers:
/// - Delay timer: General-purpose timer that decrements at 60Hz
/// - Sound timer: Controls sound output, decrements at 60Hz, produces sound when non-zero
pub struct Timers {
    /// Delay timer value (8-bit).
    delay_timer: u8,
    
    /// Sound timer value (8-bit).
    sound_timer: u8,
    
    /// Last time the timers were updated.
    last_update: Instant,
    
    /// Accumulated time since last timer decrement.
    accumulated_time: Duration,
}

impl Timers {
    /// Creates a new timer system.
    pub fn new() -> Self {
        Self {
            delay_timer: 0,
            sound_timer: 0,
            last_update: Instant::now(),
            accumulated_time: Duration::new(0, 0),
        }
    }

    /// Resets both timers to zero and timing state.
    pub fn reset(&mut self) {
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.last_update = Instant::now();
        self.accumulated_time = Duration::new(0, 0);
    }

    /// Gets the current delay timer value.
    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer
    }

    /// Sets the delay timer value.
    /// 
    /// # Arguments
    /// * `value` - Timer value (0-255)
    pub fn set_delay_timer(&mut self, value: u8) {
        self.delay_timer = value;
    }

    /// Gets the current sound timer value.
    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer
    }

    /// Sets the sound timer value.
    /// 
    /// # Arguments
    /// * `value` - Timer value (0-255)
    pub fn set_sound_timer(&mut self, value: u8) {
        self.sound_timer = value;
    }

    /// Checks if sound should be playing (sound timer > 0).
    pub fn should_play_sound(&self) -> bool {
        self.sound_timer > 0
    }

    /// Updates both timers based on elapsed time.
    /// 
    /// This should be called regularly (ideally every emulation cycle)
    /// to maintain accurate 60Hz timing.
    /// 
    /// # Returns
    /// True if either timer was decremented, false otherwise.
    pub fn update(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);
        self.last_update = now;
        
        self.accumulated_time += elapsed;
        
        let timer_period = Duration::from_millis(TIMER_PERIOD_MS);
        let mut timers_decremented = false;
        
        // Decrement timers for each complete timer period that has elapsed
        while self.accumulated_time >= timer_period {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
                timers_decremented = true;
            }
            
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
                timers_decremented = true;
            }
            
            self.accumulated_time -= timer_period;
        }
        
        timers_decremented
    }

    /// Updates timers by a specific number of ticks.
    /// 
    /// This is useful for testing or when you want to manually control
    /// timer progression rather than using real time.
    /// 
    /// # Arguments
    /// * `ticks` - Number of timer ticks to advance
    /// 
    /// # Returns
    /// True if either timer was decremented, false otherwise.
    pub fn update_by_ticks(&mut self, ticks: u32) -> bool {
        let mut timers_decremented = false;
        
        for _ in 0..ticks {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
                timers_decremented = true;
            }
            
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
                timers_decremented = true;
            }
        }
        
        timers_decremented
    }

    /// Gets the time remaining until the next timer update.
    /// 
    /// This is useful for determining when to schedule the next update
    /// or for debugging timing issues.
    pub fn time_until_next_update(&self) -> Duration {
        let timer_period = Duration::from_millis(TIMER_PERIOD_MS);
        
        if self.accumulated_time >= timer_period {
            Duration::new(0, 0) // Update needed now
        } else {
            timer_period - self.accumulated_time
        }
    }

    /// Gets the number of timer ticks that have occurred since creation or reset.
    /// 
    /// This is calculated based on elapsed real time and can be useful for
    /// performance monitoring and debugging.
    pub fn total_ticks_elapsed(&self) -> u64 {
        let total_time = self.last_update.duration_since(
            self.last_update - self.accumulated_time - Duration::from_secs(0) // Approximation
        );
        
        total_time.as_millis() as u64 / TIMER_PERIOD_MS
    }

    /// Synchronizes the timer update rate to maintain 60Hz accuracy.
    /// 
    /// This method can be used to keep the timers synchronized with
    /// external timing sources or to correct for drift.
    pub fn sync_to_real_time(&mut self) {
        self.last_update = Instant::now();
        self.accumulated_time = Duration::new(0, 0);
    }

    /// Gets both timer values as a tuple for convenience.
    /// 
    /// # Returns
    /// (delay_timer, sound_timer)
    pub fn get_both_timers(&self) -> (u8, u8) {
        (self.delay_timer, self.sound_timer)
    }

    /// Sets both timer values at once.
    /// 
    /// # Arguments
    /// * `delay_value` - Delay timer value
    /// * `sound_value` - Sound timer value
    pub fn set_both_timers(&mut self, delay_value: u8, sound_value: u8) {
        self.delay_timer = delay_value;
        self.sound_timer = sound_value;
    }

    /// Checks if both timers are zero (inactive).
    pub fn are_both_timers_zero(&self) -> bool {
        self.delay_timer == 0 && self.sound_timer == 0
    }

    /// Gets the current timer frequency for verification.
    pub fn get_frequency(&self) -> u32 {
        TIMER_FREQUENCY
    }

    /// Gets timing statistics for debugging and profiling.
    pub fn get_timing_stats(&self) -> TimingStats {
        TimingStats {
            delay_timer: self.delay_timer,
            sound_timer: self.sound_timer,
            accumulated_time_ms: self.accumulated_time.as_millis() as u64,
            time_until_next_update_ms: self.time_until_next_update().as_millis() as u64,
            frequency_hz: TIMER_FREQUENCY,
            period_ms: TIMER_PERIOD_MS,
        }
    }
}

impl Default for Timers {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer statistics for debugging and profiling.
#[derive(Debug, Clone, PartialEq)]
pub struct TimingStats {
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub accumulated_time_ms: u64,
    pub time_until_next_update_ms: u64,
    pub frequency_hz: u32,
    pub period_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_timer_creation() {
        let timers = Timers::new();
        
        assert_eq!(timers.get_delay_timer(), 0);
        assert_eq!(timers.get_sound_timer(), 0);
        assert!(!timers.should_play_sound());
        assert!(timers.are_both_timers_zero());
    }

    #[test]
    fn test_timer_set_get() {
        let mut timers = Timers::new();
        
        timers.set_delay_timer(100);
        timers.set_sound_timer(50);
        
        assert_eq!(timers.get_delay_timer(), 100);
        assert_eq!(timers.get_sound_timer(), 50);
        assert!(timers.should_play_sound());
        assert!(!timers.are_both_timers_zero());
        
        let (delay, sound) = timers.get_both_timers();
        assert_eq!(delay, 100);
        assert_eq!(sound, 50);
    }

    #[test]
    fn test_timer_ticks() {
        let mut timers = Timers::new();
        
        timers.set_delay_timer(10);
        timers.set_sound_timer(5);
        
        // Update by 3 ticks
        let changed = timers.update_by_ticks(3);
        assert!(changed);
        assert_eq!(timers.get_delay_timer(), 7);
        assert_eq!(timers.get_sound_timer(), 2);
        
        // Update by 2 more ticks - sound timer should reach zero
        timers.update_by_ticks(2);
        assert_eq!(timers.get_delay_timer(), 5);
        assert_eq!(timers.get_sound_timer(), 0);
        assert!(!timers.should_play_sound());
        
        // Update beyond delay timer value
        timers.update_by_ticks(10);
        assert_eq!(timers.get_delay_timer(), 0);
        assert_eq!(timers.get_sound_timer(), 0);
        assert!(timers.are_both_timers_zero());
    }

    #[test]
    fn test_timer_no_decrement_when_zero() {
        let mut timers = Timers::new();
        
        // Timers start at zero
        let changed = timers.update_by_ticks(5);
        assert!(!changed); // No change should occur
        assert_eq!(timers.get_delay_timer(), 0);
        assert_eq!(timers.get_sound_timer(), 0);
    }

    #[test]
    fn test_both_timers_operations() {
        let mut timers = Timers::new();
        
        timers.set_both_timers(200, 100);
        assert_eq!(timers.get_both_timers(), (200, 100));
        
        timers.set_both_timers(0, 0);
        assert!(timers.are_both_timers_zero());
    }

    #[test]
    fn test_reset() {
        let mut timers = Timers::new();
        
        timers.set_delay_timer(255);
        timers.set_sound_timer(128);
        
        timers.reset();
        
        assert_eq!(timers.get_delay_timer(), 0);
        assert_eq!(timers.get_sound_timer(), 0);
        assert!(timers.are_both_timers_zero());
    }

    #[test]
    fn test_timing_constants() {
        assert_eq!(TIMER_FREQUENCY, 60);
        assert_eq!(TIMER_PERIOD_MS, 16); // 1000ms / 60Hz ≈ 16.67ms, truncated to 16
        
        let timers = Timers::new();
        assert_eq!(timers.get_frequency(), 60);
    }

    #[test]
    fn test_sound_timer_behavior() {
        let mut timers = Timers::new();
        
        // Sound should not play when timer is zero
        assert!(!timers.should_play_sound());
        
        // Sound should play when timer is non-zero
        timers.set_sound_timer(1);
        assert!(timers.should_play_sound());
        
        // Sound should stop when timer reaches zero
        timers.update_by_ticks(1);
        assert!(!timers.should_play_sound());
    }

    #[test]
    fn test_timing_stats() {
        let mut timers = Timers::new();
        timers.set_delay_timer(100);
        timers.set_sound_timer(50);
        
        let stats = timers.get_timing_stats();
        
        assert_eq!(stats.delay_timer, 100);
        assert_eq!(stats.sound_timer, 50);
        assert_eq!(stats.frequency_hz, 60);
        assert_eq!(stats.period_ms, 16);
    }

    #[test]
    fn test_real_time_update() {
        let mut timers = Timers::new();
        timers.set_delay_timer(60); // Should last about 1 second at 60Hz
        timers.set_sound_timer(30); // Should last about 0.5 seconds at 60Hz
        
        // Sleep for a short time to allow some timer updates
        thread::sleep(Duration::from_millis(50)); // Sleep for 50ms
        
        let updated = timers.update();
        
        // With 50ms sleep, we should have had 2-3 timer ticks (50ms / 16.67ms ≈ 3)
        // So timers should have decremented but not reached zero
        if updated {
            assert!(timers.get_delay_timer() < 60);
            assert!(timers.get_sound_timer() < 30);
            assert!(timers.get_delay_timer() > 55); // Shouldn't have decremented too much
            assert!(timers.get_sound_timer() > 25);
        }
    }

    #[test]
    fn test_time_until_next_update() {
        let mut timers = Timers::new();
        
        // Just created, should need update soon
        let time_until = timers.time_until_next_update();
        assert!(time_until <= Duration::from_millis(TIMER_PERIOD_MS));
        
        // After sync, should reset timing
        timers.sync_to_real_time();
        let time_until = timers.time_until_next_update();
        assert_eq!(time_until, Duration::from_millis(TIMER_PERIOD_MS));
    }

    #[test]
    fn test_edge_cases() {
        let mut timers = Timers::new();
        
        // Test maximum timer values
        timers.set_delay_timer(255);
        timers.set_sound_timer(255);
        
        timers.update_by_ticks(1);
        assert_eq!(timers.get_delay_timer(), 254);
        assert_eq!(timers.get_sound_timer(), 254);
        
        // Test timer underflow protection
        timers.set_delay_timer(1);
        timers.set_sound_timer(1);
        
        timers.update_by_ticks(5); // More ticks than timer values
        assert_eq!(timers.get_delay_timer(), 0);
        assert_eq!(timers.get_sound_timer(), 0);
    }

    #[test]
    fn test_independent_timer_behavior() {
        let mut timers = Timers::new();
        
        // Set different values and verify they decrement independently
        timers.set_delay_timer(10);
        timers.set_sound_timer(3);
        
        timers.update_by_ticks(3);
        assert_eq!(timers.get_delay_timer(), 7);
        assert_eq!(timers.get_sound_timer(), 0);
        
        // Continue updating, only delay timer should change
        timers.update_by_ticks(2);
        assert_eq!(timers.get_delay_timer(), 5);
        assert_eq!(timers.get_sound_timer(), 0);
    }
}