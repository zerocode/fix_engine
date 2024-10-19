pub trait Clock: Send + Sync {
    fn now(&self) -> String;
}

#[derive(Debug)]
pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> String {
        let now = chrono::Utc::now();
        format!("{}", now.format("%Y%m%d-%H:%M:%S%.3f"))
    }
}