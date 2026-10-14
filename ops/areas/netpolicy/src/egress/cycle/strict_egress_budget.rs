use super::*;

use acme_kit::text;

/// Where a strict egress budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictEgressBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictEgressBudgetStatus {
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

impl std::fmt::Display for StrictEgressBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict egress budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressBudget {
    pub code: String,
    pub label: String,
    pub name: String,
    pub timezone: String,
    pub host: String,
    pub cipher: String,
    pub contact_email: String,
    pub minimum: f64,
    pub currency: String,
    pub revision: i64,
    pub key_id: String,
    pub status: StrictEgressBudgetStatus,
}

impl StrictEgressBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictEgressBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictEgressBudgetStatus) -> bool {
        match self.status {
            StrictEgressBudgetStatus::Draft => {
                return next == StrictEgressBudgetStatus::Suspended;
            }
            StrictEgressBudgetStatus::Pending => {
                match next {
                    StrictEgressBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEgressBudgetStatus::Active => {
                return next == StrictEgressBudgetStatus::Suspended || next == StrictEgressBudgetStatus::Closed;
            }
            StrictEgressBudgetStatus::Suspended => {
                return next == StrictEgressBudgetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictEgressBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict egress budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.revision as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictEgressBudget {
        StrictEgressBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            host: self.host.clone(),
            cipher: self.cipher.clone(),
            contact_email: self.contact_email.clone(),
            minimum: self.minimum,
            currency: self.currency.clone(),
            revision: self.revision,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict egress budget in status draft.
pub fn new_strict_egress_budget(code: &str, label: &str) -> StrictEgressBudget {
    StrictEgressBudget {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        timezone: "".to_string(),
        host: "".to_string(),
        cipher: "".to_string(),
        contact_email: "".to_string(),
        minimum: 0.0,
        currency: "".to_string(),
        revision: 0,
        key_id: "".to_string(),
        status: StrictEgressBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_egress_budget("code-583", "label-957");
        assert_eq!(item.status, StrictEgressBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_strict_egress_budget("code-583", "label-957");
        if item.move_to(StrictEgressBudgetStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictEgressBudgetStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_egress_budget("code-583", "label-957");
        item.status = StrictEgressBudgetStatus::Closed;
        assert!(
            item.move_to(StrictEgressBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
