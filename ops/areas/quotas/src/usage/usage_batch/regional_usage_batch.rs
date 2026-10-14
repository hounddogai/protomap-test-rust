use super::*;

use acme_kit::text;

/// Where a regional usage batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalUsageBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalUsageBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalUsageBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional usage batch as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageBatch {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub amount: f64,
    pub notes: String,
    pub external_ref: String,
    pub source: String,
    pub window_days: i64,
    pub status: RegionalUsageBatchStatus,
}

impl RegionalUsageBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalUsageBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalUsageBatchStatus) -> bool {
        match self.status {
            RegionalUsageBatchStatus::Proposed => {
                return next == RegionalUsageBatchStatus::Approved || next == RegionalUsageBatchStatus::Rejected;
            }
            RegionalUsageBatchStatus::Approved => {
                match next {
                    RegionalUsageBatchStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalUsageBatchStatus::Rejected => {
                match next {
                    RegionalUsageBatchStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionalUsageBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional usage batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional usage batch", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.amount + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalUsageBatch {
        RegionalUsageBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            amount: self.amount,
            notes: self.notes.clone(),
            external_ref: self.external_ref.clone(),
            source: self.source.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a regional usage batch in status proposed.
pub fn new_regional_usage_batch(code: &str, name: &str) -> RegionalUsageBatch {
    RegionalUsageBatch {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        amount: 0.0,
        notes: "".to_string(),
        external_ref: "".to_string(),
        source: "".to_string(),
        window_days: 0,
        status: RegionalUsageBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_usage_batch("code-390", "name-881");
        assert_eq!(item.status, RegionalUsageBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_regional_usage_batch("code-390", "name-881");
        if item.move_to(RegionalUsageBatchStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalUsageBatchStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_usage_batch("code-390", "name-881");
        item.status = RegionalUsageBatchStatus::Withdrawn;
        assert!(
            item.move_to(RegionalUsageBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
