use super::*;

use acme_kit::text;

/// Where a tiered credential plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredCredentialPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredCredentialPlanStatus {
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

impl std::fmt::Display for TieredCredentialPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered credential plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPlan {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub category: String,
    pub priority: i64,
    pub approved: bool,
    pub margin: f64,
    pub cluster: String,
    pub status: TieredCredentialPlanStatus,
}

impl TieredCredentialPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredCredentialPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredCredentialPlanStatus) -> bool {
        match self.status {
            TieredCredentialPlanStatus::Open => {
                return next == TieredCredentialPlanStatus::InReview || next == TieredCredentialPlanStatus::Resolved;
            }
            TieredCredentialPlanStatus::InReview => {
                return next == TieredCredentialPlanStatus::Reopened;
            }
            TieredCredentialPlanStatus::Resolved => {
                match next {
                    TieredCredentialPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredCredentialPlanStatus::Reopened => {
                return next == TieredCredentialPlanStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredCredentialPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered credential plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.priority as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredCredentialPlan {
        TieredCredentialPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            category: self.category.clone(),
            priority: self.priority,
            approved: self.approved,
            margin: self.margin,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tiered credential plan in status open.
pub fn new_tiered_credential_plan(code: &str, name: &str) -> TieredCredentialPlan {
    TieredCredentialPlan {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        category: "".to_string(),
        priority: 0,
        approved: false,
        margin: 0.0,
        cluster: "".to_string(),
        status: TieredCredentialPlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_credential_plan("code-400", "name-170");
        assert_eq!(item.status, TieredCredentialPlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_tiered_credential_plan("code-400", "name-170");
        if item.move_to(TieredCredentialPlanStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredCredentialPlanStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_credential_plan("code-400", "name-170");
        item.status = TieredCredentialPlanStatus::Archived;
        assert!(
            item.move_to(TieredCredentialPlanStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
