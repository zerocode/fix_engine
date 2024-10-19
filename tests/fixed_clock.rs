use std::sync::Arc;
use fix_engine_2::clock::Clock;

// A FixedClock for testing purposes
pub struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> String {
        "20231016-12:30:00.123".to_string() // Fixed timestamp for testing
    }
}

// Helper function to provide Arc<dyn Clock>
pub fn create_fixed_clock() -> Arc<dyn Clock> {
    Arc::new(FixedClock)
}
