use super::*;

use acme_kit::text;

/// Where a default rule review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRuleReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultRuleReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DefaultRuleReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default rule review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleReview {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub threshold: f64,
    pub starts_at: i64,
    pub contact_email: String,
    pub rate: f64,
    pub status: DefaultRuleReviewStatus,
}

impl DefaultRuleReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRuleReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRuleReviewStatus) -> bool {
        match self.status {
            DefaultRuleReviewStatus::Open => {
                return next == DefaultRuleReviewStatus::Resolved || next == DefaultRuleReviewStatus::Reopened;
            }
            DefaultRuleReviewStatus::InReview => {
                match next {
                    DefaultRuleReviewStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRuleReviewStatus::Resolved => {
                match next {
                    DefaultRuleReviewStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRuleReviewStatus::Reopened => {
                match next {
                    DefaultRuleReviewStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultRuleReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rule review {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.threshold - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRuleReview {
        DefaultRuleReview {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            threshold: self.threshold,
            starts_at: self.starts_at,
            contact_email: self.contact_email.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a default rule review in status open.
pub fn new_default_rule_review(code: &str, label: &str) -> DefaultRuleReview {
    DefaultRuleReview {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        threshold: 0.0,
        starts_at: 0,
        contact_email: "".to_string(),
        rate: 0.0,
        status: DefaultRuleReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_rule_review("code-403", "label-670");
        assert_eq!(item.status, DefaultRuleReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_default_rule_review("code-403", "label-670");
        if item.move_to(DefaultRuleReviewStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRuleReviewStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_rule_review("code-403", "label-670");
        item.status = DefaultRuleReviewStatus::Archived;
        assert!(
            item.move_to(DefaultRuleReviewStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
