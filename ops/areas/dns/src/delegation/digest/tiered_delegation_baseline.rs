use super::*;

use acme_kit::text;

/// Where a tiered delegation baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDelegationBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredDelegationBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TieredDelegationBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered delegation baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationBaseline {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub maximum: f64,
    pub issuer: String,
    pub quantity: i64,
    pub summary: String,
    pub category: String,
    pub ceiling: f64,
    pub namespace: String,
    pub status: TieredDelegationBaselineStatus,
}

impl TieredDelegationBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDelegationBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDelegationBaselineStatus) -> bool {
        match self.status {
            TieredDelegationBaselineStatus::Planned => {
                return next == TieredDelegationBaselineStatus::InProgress
                    || next == TieredDelegationBaselineStatus::Completed;
            }
            TieredDelegationBaselineStatus::Scheduled => {
                return next == TieredDelegationBaselineStatus::InProgress;
            }
            TieredDelegationBaselineStatus::InProgress => {
                return next == TieredDelegationBaselineStatus::Cancelled;
            }
            TieredDelegationBaselineStatus::Completed => {
                return next == TieredDelegationBaselineStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredDelegationBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered delegation baseline {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.starts_at as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDelegationBaseline {
        TieredDelegationBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            maximum: self.maximum,
            issuer: self.issuer.clone(),
            quantity: self.quantity,
            summary: self.summary.clone(),
            category: self.category.clone(),
            ceiling: self.ceiling,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered delegation baseline in status planned.
pub fn new_tiered_delegation_baseline(code: &str, name: &str) -> TieredDelegationBaseline {
    TieredDelegationBaseline {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        maximum: 0.0,
        issuer: "".to_string(),
        quantity: 0,
        summary: "".to_string(),
        category: "".to_string(),
        ceiling: 0.0,
        namespace: "".to_string(),
        status: TieredDelegationBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_delegation_baseline("code-951", "name-805");
        assert_eq!(item.status, TieredDelegationBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_tiered_delegation_baseline("code-951", "name-805");
        if item.move_to(TieredDelegationBaselineStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDelegationBaselineStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_delegation_baseline("code-951", "name-805");
        item.status = TieredDelegationBaselineStatus::Cancelled;
        assert!(
            item.move_to(TieredDelegationBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
