use super::*;

use acme_kit::text;

/// Where a quote policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuotePolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QuotePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QuotePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QuotePolicy {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub amount: f64,
    pub description: String,
    pub version: i64,
    pub duration_minutes: i64,
    pub currency: String,
    pub notes: String,
    pub status: QuotePolicyStatus,
}

impl QuotePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuotePolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuotePolicyStatus) -> bool {
        match self.status {
            QuotePolicyStatus::Proposed => {
                return next == QuotePolicyStatus::Rejected;
            }
            QuotePolicyStatus::Approved => {
                match next {
                    QuotePolicyStatus::Rejected | QuotePolicyStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuotePolicyStatus::Rejected => {
                match next {
                    QuotePolicyStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: QuotePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.version as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuotePolicy {
        QuotePolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            amount: self.amount,
            description: self.description.clone(),
            version: self.version,
            duration_minutes: self.duration_minutes,
            currency: self.currency.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a quote policy in status proposed.
pub fn new_quote_policy(code: &str, name: &str) -> QuotePolicy {
    QuotePolicy {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        amount: 0.0,
        description: "".to_string(),
        version: 0,
        duration_minutes: 0,
        currency: "".to_string(),
        notes: "".to_string(),
        status: QuotePolicyStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_quote_policy("code-945", "name-946");
        assert_eq!(item.status, QuotePolicyStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_quote_policy("code-945", "name-946");
        if item.move_to(QuotePolicyStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuotePolicyStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_quote_policy("code-945", "name-946");
        item.status = QuotePolicyStatus::Withdrawn;
        assert!(
            item.move_to(QuotePolicyStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
