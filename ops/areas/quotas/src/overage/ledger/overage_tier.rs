use super::*;

use acme_kit::text;

/// Where a overage tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl OverageTierStatus {
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

impl std::fmt::Display for OverageTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageTier {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub pinned: bool,
    pub expires_at: i64,
    pub amount: f64,
    pub ceiling: f64,
    pub status: OverageTierStatus,
}

impl OverageTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageTierStatus) -> bool {
        match self.status {
            OverageTierStatus::New => {
                return next == OverageTierStatus::Running;
            }
            OverageTierStatus::Queued => {
                return next == OverageTierStatus::Running || next == OverageTierStatus::Done;
            }
            OverageTierStatus::Running => {
                return next == OverageTierStatus::Done || next == OverageTierStatus::Failed;
            }
            OverageTierStatus::Done => {
                return next == OverageTierStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "overage tier", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ceiling - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageTier {
        OverageTier {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            pinned: self.pinned,
            expires_at: self.expires_at,
            amount: self.amount,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a overage tier in status new.
pub fn new_overage_tier(code: &str, label: &str) -> OverageTier {
    OverageTier {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        pinned: false,
        expires_at: 0,
        amount: 0.0,
        ceiling: 0.0,
        status: OverageTierStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_overage_tier("code-956", "label-794");
        assert_eq!(item.status, OverageTierStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_overage_tier("code-956", "label-794");
        if item.move_to(OverageTierStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageTierStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_overage_tier("code-956", "label-794");
        item.status = OverageTierStatus::Failed;
        assert!(item.move_to(OverageTierStatus::New).is_err(), "expected move to fail");
    }
}
