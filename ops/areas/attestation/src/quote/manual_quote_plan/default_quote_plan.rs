use super::*;

use acme_kit::text;

/// Where a default quote plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultQuotePlanStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultQuotePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultQuotePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default quote plan as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuotePlan {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub name: String,
    pub key_id: String,
    pub source: String,
    pub checksum: String,
    pub score: f64,
    pub status: DefaultQuotePlanStatus,
}

impl DefaultQuotePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultQuotePlanStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultQuotePlanStatus) -> bool {
        match self.status {
            DefaultQuotePlanStatus::Proposed => {
                match next {
                    DefaultQuotePlanStatus::Approved | DefaultQuotePlanStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultQuotePlanStatus::Approved => {
                return next == DefaultQuotePlanStatus::Rejected;
            }
            DefaultQuotePlanStatus::Rejected => {
                return next == DefaultQuotePlanStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultQuotePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default quote plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default quote plan", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> DefaultQuotePlan {
        DefaultQuotePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            source: self.source.clone(),
            checksum: self.checksum.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default quote plan in status proposed.
pub fn new_default_quote_plan(code: &str, label: &str) -> DefaultQuotePlan {
    DefaultQuotePlan {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        name: "".to_string(),
        key_id: "".to_string(),
        source: "".to_string(),
        checksum: "".to_string(),
        score: 0.0,
        status: DefaultQuotePlanStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_quote_plan("code-461", "label-453");
        assert_eq!(item.status, DefaultQuotePlanStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_default_quote_plan("code-461", "label-453");
        if item.move_to(DefaultQuotePlanStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultQuotePlanStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_quote_plan("code-461", "label-453");
        item.status = DefaultQuotePlanStatus::Withdrawn;
        assert!(
            item.move_to(DefaultQuotePlanStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
