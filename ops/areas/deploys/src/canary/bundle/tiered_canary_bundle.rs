use super::*;

use acme_kit::text;

/// Where a tiered canary bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredCanaryBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredCanaryBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TieredCanaryBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered canary bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryBundle {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub retries: i64,
    pub port: i64,
    pub starts_at: i64,
    pub expires_at: i64,
    pub timezone: String,
    pub status: TieredCanaryBundleStatus,
}

impl TieredCanaryBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredCanaryBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredCanaryBundleStatus) -> bool {
        match self.status {
            TieredCanaryBundleStatus::Idle => {
                return next == TieredCanaryBundleStatus::Warming || next == TieredCanaryBundleStatus::Ready;
            }
            TieredCanaryBundleStatus::Warming => {
                match next {
                    TieredCanaryBundleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredCanaryBundleStatus::Ready => {
                match next {
                    TieredCanaryBundleStatus::Draining | TieredCanaryBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredCanaryBundleStatus::Draining => {
                return next == TieredCanaryBundleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredCanaryBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered canary bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered canary bundle", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.port as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredCanaryBundle {
        TieredCanaryBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            retries: self.retries,
            port: self.port,
            starts_at: self.starts_at,
            expires_at: self.expires_at,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered canary bundle in status idle.
pub fn new_tiered_canary_bundle(code: &str, name: &str) -> TieredCanaryBundle {
    TieredCanaryBundle {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        retries: 0,
        port: 0,
        starts_at: 0,
        expires_at: 0,
        timezone: "".to_string(),
        status: TieredCanaryBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_canary_bundle("code-379", "name-769");
        assert_eq!(item.status, TieredCanaryBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_tiered_canary_bundle("code-379", "name-769");
        if item.move_to(TieredCanaryBundleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredCanaryBundleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_canary_bundle("code-379", "name-769");
        item.status = TieredCanaryBundleStatus::Stopped;
        assert!(
            item.move_to(TieredCanaryBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
