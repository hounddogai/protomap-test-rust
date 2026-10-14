use super::*;

use acme_kit::text;

/// Where a rolling limit group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingLimitGroupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingLimitGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollingLimitGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling limit group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitGroup {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub quantity: i64,
    pub active: bool,
    pub window_days: i64,
    pub status: RollingLimitGroupStatus,
}

impl RollingLimitGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingLimitGroupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingLimitGroupStatus) -> bool {
        match self.status {
            RollingLimitGroupStatus::New => {
                return next == RollingLimitGroupStatus::Running || next == RollingLimitGroupStatus::Done;
            }
            RollingLimitGroupStatus::Queued => {
                return next == RollingLimitGroupStatus::Running;
            }
            RollingLimitGroupStatus::Running => {
                return next == RollingLimitGroupStatus::Done || next == RollingLimitGroupStatus::Failed;
            }
            RollingLimitGroupStatus::Done => {
                return next == RollingLimitGroupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingLimitGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling limit group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.window_days as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingLimitGroup {
        RollingLimitGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            quantity: self.quantity,
            active: self.active,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling limit group in status new.
pub fn new_rolling_limit_group(code: &str, name: &str) -> RollingLimitGroup {
    RollingLimitGroup {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        quantity: 0,
        active: false,
        window_days: 0,
        status: RollingLimitGroupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_limit_group("code-407", "name-261");
        assert_eq!(item.status, RollingLimitGroupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rolling_limit_group("code-407", "name-261");
        if item.move_to(RollingLimitGroupStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingLimitGroupStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_limit_group("code-407", "name-261");
        item.status = RollingLimitGroupStatus::Failed;
        assert!(
            item.move_to(RollingLimitGroupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
