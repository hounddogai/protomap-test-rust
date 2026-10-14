use super::*;

use acme_kit::text;

/// Where a retention baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RetentionBaselineStatus {
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

impl std::fmt::Display for RetentionBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention baseline as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBaseline {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub floor_value: f64,
    pub duration_minutes: i64,
    pub region: String,
    pub priority: i64,
    pub pinned: bool,
    pub status: RetentionBaselineStatus,
}

impl RetentionBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionBaselineStatus) -> bool {
        match self.status {
            RetentionBaselineStatus::Idle => {
                return next == RetentionBaselineStatus::Warming || next == RetentionBaselineStatus::Stopped;
            }
            RetentionBaselineStatus::Warming => {
                match next {
                    RetentionBaselineStatus::Ready | RetentionBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionBaselineStatus::Ready => {
                match next {
                    RetentionBaselineStatus::Draining | RetentionBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionBaselineStatus::Draining => {
                return next == RetentionBaselineStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.duration_minutes as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionBaseline {
        RetentionBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            floor_value: self.floor_value,
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            priority: self.priority,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a retention baseline in status idle.
pub fn new_retention_baseline(code: &str, title: &str) -> RetentionBaseline {
    RetentionBaseline {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        floor_value: 0.0,
        duration_minutes: 0,
        region: "".to_string(),
        priority: 0,
        pinned: false,
        status: RetentionBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_retention_baseline("code-389", "title-215");
        assert_eq!(item.status, RetentionBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_retention_baseline("code-389", "title-215");
        if item.move_to(RetentionBaselineStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionBaselineStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_retention_baseline("code-389", "title-215");
        item.status = RetentionBaselineStatus::Stopped;
        assert!(
            item.move_to(RetentionBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
