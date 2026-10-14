use super::*;

use acme_kit::text;

/// Where a usage target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl UsageTargetStatus {
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

impl std::fmt::Display for UsageTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageTarget {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub budget: f64,
    pub unit: String,
    pub starts_at: i64,
    pub rotation_days: i64,
    pub expires_at: i64,
    pub quota_units: i64,
    pub cluster: String,
    pub region: String,
    pub status: UsageTargetStatus,
}

impl UsageTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageTargetStatus) -> bool {
        match self.status {
            UsageTargetStatus::Idle => {
                return next == UsageTargetStatus::Warming || next == UsageTargetStatus::Draining;
            }
            UsageTargetStatus::Warming => {
                match next {
                    UsageTargetStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageTargetStatus::Ready => {
                return next == UsageTargetStatus::Stopped;
            }
            UsageTargetStatus::Draining => {
                return next == UsageTargetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage target {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.starts_at as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageTarget {
        UsageTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            budget: self.budget,
            unit: self.unit.clone(),
            starts_at: self.starts_at,
            rotation_days: self.rotation_days,
            expires_at: self.expires_at,
            quota_units: self.quota_units,
            cluster: self.cluster.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a usage target in status idle.
pub fn new_usage_target(code: &str, title: &str) -> UsageTarget {
    UsageTarget {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        budget: 0.0,
        unit: "".to_string(),
        starts_at: 0,
        rotation_days: 0,
        expires_at: 0,
        quota_units: 0,
        cluster: "".to_string(),
        region: "".to_string(),
        status: UsageTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_usage_target("code-316", "title-395");
        assert_eq!(item.status, UsageTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_usage_target("code-316", "title-395");
        if item.move_to(UsageTargetStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageTargetStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_usage_target("code-316", "title-395");
        item.status = UsageTargetStatus::Stopped;
        assert!(item.move_to(UsageTargetStatus::Idle).is_err(), "expected move to fail");
    }
}
