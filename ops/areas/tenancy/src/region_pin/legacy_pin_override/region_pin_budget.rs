use super::*;

use acme_kit::text;

/// Where a region pin budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionPinBudgetStatus {
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

impl std::fmt::Display for RegionPinBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin budget as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBudget {
    pub code: String,
    pub label: String,
    pub region: String,
    pub contact_email: String,
    pub ceiling: f64,
    pub attempts: i64,
    pub created_at: i64,
    pub limit: i64,
    pub status: RegionPinBudgetStatus,
}

impl RegionPinBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinBudgetStatus) -> bool {
        match self.status {
            RegionPinBudgetStatus::Planned => {
                match next {
                    RegionPinBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinBudgetStatus::Scheduled => {
                match next {
                    RegionPinBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinBudgetStatus::InProgress => {
                match next {
                    RegionPinBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinBudgetStatus::Completed => {
                match next {
                    RegionPinBudgetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionPinBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin budget {} cannot move from {} to {}",
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
        format!("{} {}", "region pin budget", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        (self.ceiling * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinBudget {
        RegionPinBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            contact_email: self.contact_email.clone(),
            ceiling: self.ceiling,
            attempts: self.attempts,
            created_at: self.created_at,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a region pin budget in status planned.
pub fn new_region_pin_budget(code: &str, label: &str) -> RegionPinBudget {
    RegionPinBudget {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        contact_email: "".to_string(),
        ceiling: 0.0,
        attempts: 0,
        created_at: 0,
        limit: 0,
        status: RegionPinBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_region_pin_budget("code-784", "label-607");
        assert_eq!(item.status, RegionPinBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_region_pin_budget("code-784", "label-607");
        if item.move_to(RegionPinBudgetStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinBudgetStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_region_pin_budget("code-784", "label-607");
        item.status = RegionPinBudgetStatus::Cancelled;
        assert!(
            item.move_to(RegionPinBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
