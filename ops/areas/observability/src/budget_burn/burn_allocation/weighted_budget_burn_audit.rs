use super::*;

use acme_kit::text;

/// Where a weighted budget burn audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedBudgetBurnAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedBudgetBurnAuditStatus {
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

impl std::fmt::Display for WeightedBudgetBurnAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted budget burn audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnAudit {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub key_id: String,
    pub cipher: String,
    pub duration_minutes: i64,
    pub rate: f64,
    pub ceiling: f64,
    pub created_at: i64,
    pub checksum: String,
    pub status: WeightedBudgetBurnAuditStatus,
}

impl WeightedBudgetBurnAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedBudgetBurnAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedBudgetBurnAuditStatus) -> bool {
        match self.status {
            WeightedBudgetBurnAuditStatus::Planned => {
                match next {
                    WeightedBudgetBurnAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBudgetBurnAuditStatus::Scheduled => {
                match next {
                    WeightedBudgetBurnAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBudgetBurnAuditStatus::InProgress => {
                match next {
                    WeightedBudgetBurnAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBudgetBurnAuditStatus::Completed => {
                match next {
                    WeightedBudgetBurnAuditStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: WeightedBudgetBurnAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted budget burn audit {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "weighted budget burn audit", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.created_at as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedBudgetBurnAudit {
        WeightedBudgetBurnAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            key_id: self.key_id.clone(),
            cipher: self.cipher.clone(),
            duration_minutes: self.duration_minutes,
            rate: self.rate,
            ceiling: self.ceiling,
            created_at: self.created_at,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted budget burn audit in status planned.
pub fn new_weighted_budget_burn_audit(code: &str, name: &str) -> WeightedBudgetBurnAudit {
    WeightedBudgetBurnAudit {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        key_id: "".to_string(),
        cipher: "".to_string(),
        duration_minutes: 0,
        rate: 0.0,
        ceiling: 0.0,
        created_at: 0,
        checksum: "".to_string(),
        status: WeightedBudgetBurnAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_budget_burn_audit("code-59", "name-65");
        assert_eq!(item.status, WeightedBudgetBurnAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_weighted_budget_burn_audit("code-59", "name-65");
        if item.move_to(WeightedBudgetBurnAuditStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedBudgetBurnAuditStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_budget_burn_audit("code-59", "name-65");
        item.status = WeightedBudgetBurnAuditStatus::Cancelled;
        assert!(
            item.move_to(WeightedBudgetBurnAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
