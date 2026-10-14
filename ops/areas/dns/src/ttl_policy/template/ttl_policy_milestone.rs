use super::*;

use acme_kit::text;

/// Where a ttl policy milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyMilestoneStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TtlPolicyMilestoneStatus {
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

impl std::fmt::Display for TtlPolicyMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy milestone as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMilestone {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub score: f64,
    pub category: String,
    pub contact_email: String,
    pub expires_at: i64,
    pub status: TtlPolicyMilestoneStatus,
}

impl TtlPolicyMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyMilestoneStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyMilestoneStatus) -> bool {
        match self.status {
            TtlPolicyMilestoneStatus::Open => {
                match next {
                    TtlPolicyMilestoneStatus::InReview | TtlPolicyMilestoneStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyMilestoneStatus::InReview => {
                match next {
                    TtlPolicyMilestoneStatus::Resolved | TtlPolicyMilestoneStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyMilestoneStatus::Resolved => {
                match next {
                    TtlPolicyMilestoneStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyMilestoneStatus::Reopened => {
                return next == TtlPolicyMilestoneStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy milestone {} cannot move from {} to {}",
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
        format!("{} {}", "ttl policy milestone", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.score + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyMilestone {
        TtlPolicyMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            score: self.score,
            category: self.category.clone(),
            contact_email: self.contact_email.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a ttl policy milestone in status open.
pub fn new_ttl_policy_milestone(code: &str, title: &str) -> TtlPolicyMilestone {
    TtlPolicyMilestone {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        score: 0.0,
        category: "".to_string(),
        contact_email: "".to_string(),
        expires_at: 0,
        status: TtlPolicyMilestoneStatus::Open,
    }
}
