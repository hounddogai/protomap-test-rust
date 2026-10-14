use super::*;

use acme_kit::text;

/// Where a retention group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RetentionGroupStatus {
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

impl std::fmt::Display for RetentionGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionGroup {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub batch_size: i64,
    pub amount: f64,
    pub status: RetentionGroupStatus,
}

impl RetentionGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionGroupStatus) -> bool {
        match self.status {
            RetentionGroupStatus::Idle => {
                match next {
                    RetentionGroupStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionGroupStatus::Warming => {
                match next {
                    RetentionGroupStatus::Ready | RetentionGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionGroupStatus::Ready => {
                return next == RetentionGroupStatus::Draining;
            }
            RetentionGroupStatus::Draining => {
                return next == RetentionGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.created_at as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionGroup {
        RetentionGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            batch_size: self.batch_size,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention group in status idle.
pub fn new_retention_group(code: &str, label: &str) -> RetentionGroup {
    RetentionGroup {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        batch_size: 0,
        amount: 0.0,
        status: RetentionGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_retention_group("code-395", "label-328");
        assert_eq!(item.status, RetentionGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_retention_group("code-395", "label-328");
        if item.move_to(RetentionGroupStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionGroupStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_retention_group("code-395", "label-328");
        item.status = RetentionGroupStatus::Stopped;
        assert!(
            item.move_to(RetentionGroupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
