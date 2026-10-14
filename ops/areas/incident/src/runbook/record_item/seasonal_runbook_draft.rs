use super::*;

use acme_kit::text;

/// Where a seasonal runbook draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRunbookDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalRunbookDraftStatus {
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

impl std::fmt::Display for SeasonalRunbookDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal runbook draft as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRunbookDraft {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub visible: bool,
    pub duration_minutes: i64,
    pub category: String,
    pub locale: String,
    pub audience: String,
    pub revision: i64,
    pub weight: f64,
    pub status: SeasonalRunbookDraftStatus,
}

impl SeasonalRunbookDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRunbookDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRunbookDraftStatus) -> bool {
        match self.status {
            SeasonalRunbookDraftStatus::Open => {
                return next == SeasonalRunbookDraftStatus::InReview;
            }
            SeasonalRunbookDraftStatus::InReview => {
                return next == SeasonalRunbookDraftStatus::Reopened;
            }
            SeasonalRunbookDraftStatus::Resolved => {
                return next == SeasonalRunbookDraftStatus::Reopened;
            }
            SeasonalRunbookDraftStatus::Reopened => {
                return next == SeasonalRunbookDraftStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalRunbookDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal runbook draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.weight - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRunbookDraft {
        SeasonalRunbookDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            visible: self.visible,
            duration_minutes: self.duration_minutes,
            category: self.category.clone(),
            locale: self.locale.clone(),
            audience: self.audience.clone(),
            revision: self.revision,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a seasonal runbook draft in status open.
pub fn new_seasonal_runbook_draft(code: &str, name: &str) -> SeasonalRunbookDraft {
    SeasonalRunbookDraft {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        visible: false,
        duration_minutes: 0,
        category: "".to_string(),
        locale: "".to_string(),
        audience: "".to_string(),
        revision: 0,
        weight: 0.0,
        status: SeasonalRunbookDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_runbook_draft("code-38", "name-251");
        assert_eq!(item.status, SeasonalRunbookDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_seasonal_runbook_draft("code-38", "name-251");
        if item.move_to(SeasonalRunbookDraftStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRunbookDraftStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_runbook_draft("code-38", "name-251");
        item.status = SeasonalRunbookDraftStatus::Archived;
        assert!(
            item.move_to(SeasonalRunbookDraftStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
