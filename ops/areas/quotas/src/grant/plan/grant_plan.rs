use super::*;

use acme_kit::text;

/// Where a grant plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GrantPlanStatus {
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

impl std::fmt::Display for GrantPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantPlan {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub unit: String,
    pub floor_value: f64,
    pub ceiling: f64,
    pub scope: String,
    pub namespace: String,
    pub minimum: f64,
    pub name: String,
    pub expires_at: i64,
    pub status: GrantPlanStatus,
}

impl GrantPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantPlanStatus) -> bool {
        match self.status {
            GrantPlanStatus::Open => {
                match next {
                    GrantPlanStatus::Resolved | GrantPlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantPlanStatus::InReview => {
                return next == GrantPlanStatus::Resolved;
            }
            GrantPlanStatus::Resolved => {
                match next {
                    GrantPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantPlanStatus::Reopened => {
                match next {
                    GrantPlanStatus::Archived => {
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

    pub fn move_to(&mut self, next: GrantPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantPlan {
        GrantPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            unit: self.unit.clone(),
            floor_value: self.floor_value,
            ceiling: self.ceiling,
            scope: self.scope.clone(),
            namespace: self.namespace.clone(),
            minimum: self.minimum,
            name: self.name.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a grant plan in status open.
pub fn new_grant_plan(code: &str, title: &str) -> GrantPlan {
    GrantPlan {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        unit: "".to_string(),
        floor_value: 0.0,
        ceiling: 0.0,
        scope: "".to_string(),
        namespace: "".to_string(),
        minimum: 0.0,
        name: "".to_string(),
        expires_at: 0,
        status: GrantPlanStatus::Open,
    }
}
