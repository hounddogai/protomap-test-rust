use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback dashboard rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDashboardRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackDashboardRollupStatus {
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

impl std::fmt::Display for FallbackDashboardRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback dashboard rollup as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardRollup {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub minimum: f64,
    pub priority: i64,
    pub notes: String,
    pub archived: bool,
    pub batch_size: i64,
    pub status: FallbackDashboardRollupStatus,
}

impl FallbackDashboardRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDashboardRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDashboardRollupStatus) -> bool {
        match self.status {
            FallbackDashboardRollupStatus::Open => {
                match next {
                    FallbackDashboardRollupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDashboardRollupStatus::InReview => {
                match next {
                    FallbackDashboardRollupStatus::Resolved | FallbackDashboardRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDashboardRollupStatus::Resolved => {
                return next == FallbackDashboardRollupStatus::Archived;
            }
            FallbackDashboardRollupStatus::Reopened => {
                match next {
                    FallbackDashboardRollupStatus::Archived => {
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

    pub fn move_to(&mut self, next: FallbackDashboardRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback dashboard rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackDashboardRollup {
        FallbackDashboardRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            minimum: self.minimum,
            priority: self.priority,
            notes: self.notes.clone(),
            archived: self.archived,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback dashboard rollup in status open.
pub fn new_fallback_dashboard_rollup(code: &str, title: &str) -> FallbackDashboardRollup {
    FallbackDashboardRollup {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        minimum: 0.0,
        priority: 0,
        notes: "".to_string(),
        archived: false,
        batch_size: 0,
        status: FallbackDashboardRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_dashboard_rollup("code-425", "title-420");
        assert_eq!(item.status, FallbackDashboardRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_fallback_dashboard_rollup("code-425", "title-420");
        if item.move_to(FallbackDashboardRollupStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDashboardRollupStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_dashboard_rollup("code-425", "title-420");
        item.status = FallbackDashboardRollupStatus::Archived;
        assert!(
            item.move_to(FallbackDashboardRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
