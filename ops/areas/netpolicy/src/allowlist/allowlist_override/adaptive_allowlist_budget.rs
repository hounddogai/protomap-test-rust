use super::*;

use acme_kit::text;

/// Where a adaptive allowlist budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAllowlistBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveAllowlistBudgetStatus {
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

impl std::fmt::Display for AdaptiveAllowlistBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive allowlist budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBudget {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub contact_email: String,
    pub retries: i64,
    pub starts_at: i64,
    pub budget: f64,
    pub status: AdaptiveAllowlistBudgetStatus,
}

impl AdaptiveAllowlistBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAllowlistBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAllowlistBudgetStatus) -> bool {
        match self.status {
            AdaptiveAllowlistBudgetStatus::Open => {
                return next == AdaptiveAllowlistBudgetStatus::InReview
                    || next == AdaptiveAllowlistBudgetStatus::Reopened;
            }
            AdaptiveAllowlistBudgetStatus::InReview => {
                match next {
                    AdaptiveAllowlistBudgetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistBudgetStatus::Resolved => {
                match next {
                    AdaptiveAllowlistBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistBudgetStatus::Reopened => {
                return next == AdaptiveAllowlistBudgetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveAllowlistBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive allowlist budget {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive allowlist budget", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.starts_at as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAllowlistBudget {
        AdaptiveAllowlistBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            contact_email: self.contact_email.clone(),
            retries: self.retries,
            starts_at: self.starts_at,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive allowlist budget in status open.
pub fn new_adaptive_allowlist_budget(code: &str, title: &str) -> AdaptiveAllowlistBudget {
    AdaptiveAllowlistBudget {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        contact_email: "".to_string(),
        retries: 0,
        starts_at: 0,
        budget: 0.0,
        status: AdaptiveAllowlistBudgetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_allowlist_budget("code-515", "title-50");
        assert_eq!(item.status, AdaptiveAllowlistBudgetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_adaptive_allowlist_budget("code-515", "title-50");
        if item.move_to(AdaptiveAllowlistBudgetStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveAllowlistBudgetStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_allowlist_budget("code-515", "title-50");
        item.status = AdaptiveAllowlistBudgetStatus::Archived;
        assert!(
            item.move_to(AdaptiveAllowlistBudgetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
