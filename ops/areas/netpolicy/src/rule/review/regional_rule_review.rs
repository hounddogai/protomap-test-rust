use super::*;

use acme_kit::text;

/// Where a regional rule review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRuleReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalRuleReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalRuleReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional rule review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleReview {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub contact_email: String,
    pub quantity: i64,
    pub name: String,
    pub cost: f64,
    pub status: RegionalRuleReviewStatus,
}

impl RegionalRuleReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRuleReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRuleReviewStatus) -> bool {
        match self.status {
            RegionalRuleReviewStatus::Created => {
                match next {
                    RegionalRuleReviewStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRuleReviewStatus::Validated => {
                return next == RegionalRuleReviewStatus::Published || next == RegionalRuleReviewStatus::Retired;
            }
            RegionalRuleReviewStatus::Published => {
                return next == RegionalRuleReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRuleReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional rule review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional rule review", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.cost * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRuleReview {
        RegionalRuleReview {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            contact_email: self.contact_email.clone(),
            quantity: self.quantity,
            name: self.name.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional rule review in status created.
pub fn new_regional_rule_review(code: &str, label: &str) -> RegionalRuleReview {
    RegionalRuleReview {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        contact_email: "".to_string(),
        quantity: 0,
        name: "".to_string(),
        cost: 0.0,
        status: RegionalRuleReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_rule_review("code-446", "label-871");
        assert_eq!(item.status, RegionalRuleReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_regional_rule_review("code-446", "label-871");
        if item.move_to(RegionalRuleReviewStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRuleReviewStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_rule_review("code-446", "label-871");
        item.status = RegionalRuleReviewStatus::Retired;
        assert!(
            item.move_to(RegionalRuleReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
