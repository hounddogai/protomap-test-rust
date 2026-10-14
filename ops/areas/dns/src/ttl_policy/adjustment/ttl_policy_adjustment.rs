use super::*;

use acme_kit::text;

/// Where a ttl policy adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TtlPolicyAdjustmentStatus {
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

impl std::fmt::Display for TtlPolicyAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAdjustment {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub segment: String,
    pub rotation_days: i64,
    pub status: TtlPolicyAdjustmentStatus,
}

impl TtlPolicyAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyAdjustmentStatus) -> bool {
        match self.status {
            TtlPolicyAdjustmentStatus::New => {
                return next == TtlPolicyAdjustmentStatus::Running;
            }
            TtlPolicyAdjustmentStatus::Queued => {
                return next == TtlPolicyAdjustmentStatus::Running || next == TtlPolicyAdjustmentStatus::Failed;
            }
            TtlPolicyAdjustmentStatus::Running => {
                match next {
                    TtlPolicyAdjustmentStatus::Done | TtlPolicyAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyAdjustmentStatus::Done => {
                match next {
                    TtlPolicyAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: TtlPolicyAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.rotation_days as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyAdjustment {
        TtlPolicyAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            segment: self.segment.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a ttl policy adjustment in status new.
pub fn new_ttl_policy_adjustment(code: &str, label: &str) -> TtlPolicyAdjustment {
    TtlPolicyAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        segment: "".to_string(),
        rotation_days: 0,
        status: TtlPolicyAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_ttl_policy_adjustment("code-940", "label-586");
        assert_eq!(item.status, TtlPolicyAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_ttl_policy_adjustment("code-940", "label-586");
        if item.move_to(TtlPolicyAdjustmentStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyAdjustmentStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_ttl_policy_adjustment("code-940", "label-586");
        item.status = TtlPolicyAdjustmentStatus::Failed;
        assert!(
            item.move_to(TtlPolicyAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
