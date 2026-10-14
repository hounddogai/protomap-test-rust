use super::*;

use acme_kit::text;

/// Where a manual rollback tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRollbackTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualRollbackTierStatus {
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

impl std::fmt::Display for ManualRollbackTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual rollback tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackTier {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub external_ref: String,
    pub ceiling: f64,
    pub summary: String,
    pub limit: i64,
    pub status: ManualRollbackTierStatus,
}

impl ManualRollbackTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRollbackTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRollbackTierStatus) -> bool {
        match self.status {
            ManualRollbackTierStatus::Idle => {
                match next {
                    ManualRollbackTierStatus::Ready | ManualRollbackTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRollbackTierStatus::Warming => {
                return next == ManualRollbackTierStatus::Ready || next == ManualRollbackTierStatus::Stopped;
            }
            ManualRollbackTierStatus::Ready => {
                return next == ManualRollbackTierStatus::Draining;
            }
            ManualRollbackTierStatus::Draining => {
                match next {
                    ManualRollbackTierStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualRollbackTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual rollback tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual rollback tier", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.limit as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRollbackTier {
        ManualRollbackTier {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            external_ref: self.external_ref.clone(),
            ceiling: self.ceiling,
            summary: self.summary.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a manual rollback tier in status idle.
pub fn new_manual_rollback_tier(code: &str, label: &str) -> ManualRollbackTier {
    ManualRollbackTier {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        external_ref: "".to_string(),
        ceiling: 0.0,
        summary: "".to_string(),
        limit: 0,
        status: ManualRollbackTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_rollback_tier("code-107", "label-949");
        assert_eq!(item.status, ManualRollbackTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_manual_rollback_tier("code-107", "label-949");
        if item.move_to(ManualRollbackTierStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRollbackTierStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_rollback_tier("code-107", "label-949");
        item.status = ManualRollbackTierStatus::Stopped;
        assert!(
            item.move_to(ManualRollbackTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
