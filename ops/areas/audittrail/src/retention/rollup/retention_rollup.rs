use super::*;

use acme_kit::text;

/// Where a retention rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RetentionRollupStatus {
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

impl std::fmt::Display for RetentionRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRollup {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub timezone: String,
    pub discount: f64,
    pub status: RetentionRollupStatus,
}

impl RetentionRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionRollupStatus) -> bool {
        match self.status {
            RetentionRollupStatus::Idle => {
                match next {
                    RetentionRollupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionRollupStatus::Warming => {
                return next == RetentionRollupStatus::Ready;
            }
            RetentionRollupStatus::Ready => {
                match next {
                    RetentionRollupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionRollupStatus::Draining => {
                match next {
                    RetentionRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "retention rollup", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.discount - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionRollup {
        RetentionRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            timezone: self.timezone.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a retention rollup in status idle.
pub fn new_retention_rollup(code: &str, title: &str) -> RetentionRollup {
    RetentionRollup {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        timezone: "".to_string(),
        discount: 0.0,
        status: RetentionRollupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_retention_rollup("code-557", "title-941");
        assert_eq!(item.status, RetentionRollupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_retention_rollup("code-557", "title-941");
        if item.move_to(RetentionRollupStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionRollupStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_retention_rollup("code-557", "title-941");
        item.status = RetentionRollupStatus::Stopped;
        assert!(
            item.move_to(RetentionRollupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
