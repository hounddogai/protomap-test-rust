use super::*;

use acme_kit::text;

/// Where a regional rule checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRuleCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalRuleCheckpointStatus {
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

impl std::fmt::Display for RegionalRuleCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional rule checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleCheckpoint {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub created_at: i64,
    pub quantity: i64,
    pub status: RegionalRuleCheckpointStatus,
}

impl RegionalRuleCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRuleCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRuleCheckpointStatus) -> bool {
        match self.status {
            RegionalRuleCheckpointStatus::Planned => {
                return next == RegionalRuleCheckpointStatus::InProgress;
            }
            RegionalRuleCheckpointStatus::Scheduled => {
                return next == RegionalRuleCheckpointStatus::InProgress;
            }
            RegionalRuleCheckpointStatus::InProgress => {
                match next {
                    RegionalRuleCheckpointStatus::Completed | RegionalRuleCheckpointStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRuleCheckpointStatus::Completed => {
                return next == RegionalRuleCheckpointStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRuleCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional rule checkpoint {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.created_at as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRuleCheckpoint {
        RegionalRuleCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            created_at: self.created_at,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional rule checkpoint in status planned.
pub fn new_regional_rule_checkpoint(code: &str, name: &str) -> RegionalRuleCheckpoint {
    RegionalRuleCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        created_at: 0,
        quantity: 0,
        status: RegionalRuleCheckpointStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_regional_rule_checkpoint("code-984", "name-907");
        assert_eq!(item.status, RegionalRuleCheckpointStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_regional_rule_checkpoint("code-984", "name-907");
        if item.move_to(RegionalRuleCheckpointStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRuleCheckpointStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_regional_rule_checkpoint("code-984", "name-907");
        item.status = RegionalRuleCheckpointStatus::Cancelled;
        assert!(
            item.move_to(RegionalRuleCheckpointStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
