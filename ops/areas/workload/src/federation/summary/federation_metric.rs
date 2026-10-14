use super::*;

use acme_kit::text;

/// Where a federation metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FederationMetricStatus {
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

impl std::fmt::Display for FederationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One federation metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationMetric {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub attempts: i64,
    pub scope: String,
    pub visible: bool,
    pub owner_id: String,
    pub created_at: i64,
    pub archived: bool,
    pub status: FederationMetricStatus,
}

impl FederationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationMetricStatus) -> bool {
        match self.status {
            FederationMetricStatus::Open => {
                match next {
                    FederationMetricStatus::InReview | FederationMetricStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationMetricStatus::InReview => {
                return next == FederationMetricStatus::Resolved || next == FederationMetricStatus::Reopened;
            }
            FederationMetricStatus::Resolved => {
                match next {
                    FederationMetricStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationMetricStatus::Reopened => {
                return next == FederationMetricStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation metric {} cannot move from {} to {}",
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
        format!("{} {}", "federation metric", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.attempts as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationMetric {
        FederationMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            attempts: self.attempts,
            scope: self.scope.clone(),
            visible: self.visible,
            owner_id: self.owner_id.clone(),
            created_at: self.created_at,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a federation metric in status open.
pub fn new_federation_metric(code: &str, label: &str) -> FederationMetric {
    FederationMetric {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        attempts: 0,
        scope: "".to_string(),
        visible: false,
        owner_id: "".to_string(),
        created_at: 0,
        archived: false,
        status: FederationMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_federation_metric("code-297", "label-82");
        assert_eq!(item.status, FederationMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_federation_metric("code-297", "label-82");
        if item.move_to(FederationMetricStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationMetricStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_federation_metric("code-297", "label-82");
        item.status = FederationMetricStatus::Archived;
        assert!(
            item.move_to(FederationMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
