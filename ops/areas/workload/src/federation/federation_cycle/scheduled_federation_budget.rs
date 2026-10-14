use super::*;

use acme_kit::text;

/// Where a scheduled federation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFederationBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledFederationBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ScheduledFederationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled federation budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationBudget {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub audience: String,
    pub reviewer: String,
    pub maximum: f64,
    pub phone: String,
    pub cost: f64,
    pub label: String,
    pub summary: String,
    pub sequence: i64,
    pub status: ScheduledFederationBudgetStatus,
}

impl ScheduledFederationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFederationBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFederationBudgetStatus) -> bool {
        match self.status {
            ScheduledFederationBudgetStatus::Draft => {
                return next == ScheduledFederationBudgetStatus::Pending
                    || next == ScheduledFederationBudgetStatus::Suspended;
            }
            ScheduledFederationBudgetStatus::Pending => {
                match next {
                    ScheduledFederationBudgetStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFederationBudgetStatus::Active => {
                match next {
                    ScheduledFederationBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFederationBudgetStatus::Suspended => {
                match next {
                    ScheduledFederationBudgetStatus::Closed => {
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

    pub fn move_to(&mut self, next: ScheduledFederationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled federation budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.priority as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFederationBudget {
        ScheduledFederationBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            audience: self.audience.clone(),
            reviewer: self.reviewer.clone(),
            maximum: self.maximum,
            phone: self.phone.clone(),
            cost: self.cost,
            label: self.label.clone(),
            summary: self.summary.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled federation budget in status draft.
pub fn new_scheduled_federation_budget(code: &str, title: &str) -> ScheduledFederationBudget {
    ScheduledFederationBudget {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        audience: "".to_string(),
        reviewer: "".to_string(),
        maximum: 0.0,
        phone: "".to_string(),
        cost: 0.0,
        label: "".to_string(),
        summary: "".to_string(),
        sequence: 0,
        status: ScheduledFederationBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_federation_budget("code-315", "title-623");
        assert_eq!(item.status, ScheduledFederationBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_scheduled_federation_budget("code-315", "title-623");
        if item.move_to(ScheduledFederationBudgetStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFederationBudgetStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_federation_budget("code-315", "title-623");
        item.status = ScheduledFederationBudgetStatus::Closed;
        assert!(
            item.move_to(ScheduledFederationBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
