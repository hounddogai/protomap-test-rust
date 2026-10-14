use super::*;

use acme_kit::text;

/// Where a canary score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CanaryScoreStatus {
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

impl std::fmt::Display for CanaryScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryScore {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub category: String,
    pub owner_id: String,
    pub revision: i64,
    pub budget: f64,
    pub issuer: String,
    pub priority: i64,
    pub quantity: i64,
    pub status: CanaryScoreStatus,
}

impl CanaryScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryScoreStatus) -> bool {
        match self.status {
            CanaryScoreStatus::Open => {
                return next == CanaryScoreStatus::InReview || next == CanaryScoreStatus::Resolved;
            }
            CanaryScoreStatus::InReview => {
                match next {
                    CanaryScoreStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryScoreStatus::Resolved => {
                match next {
                    CanaryScoreStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryScoreStatus::Reopened => {
                match next {
                    CanaryScoreStatus::Archived => {
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

    pub fn move_to(&mut self, next: CanaryScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.weight + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryScore {
        CanaryScore {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            category: self.category.clone(),
            owner_id: self.owner_id.clone(),
            revision: self.revision,
            budget: self.budget,
            issuer: self.issuer.clone(),
            priority: self.priority,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a canary score in status open.
pub fn new_canary_score(code: &str, label: &str) -> CanaryScore {
    CanaryScore {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        category: "".to_string(),
        owner_id: "".to_string(),
        revision: 0,
        budget: 0.0,
        issuer: "".to_string(),
        priority: 0,
        quantity: 0,
        status: CanaryScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_canary_score("code-899", "label-357");
        assert_eq!(item.status, CanaryScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_canary_score("code-899", "label-357");
        if item.move_to(CanaryScoreStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryScoreStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_canary_score("code-899", "label-357");
        item.status = CanaryScoreStatus::Archived;
        assert!(item.move_to(CanaryScoreStatus::Open).is_err(), "expected move to fail");
    }
}
