use super::*;

use acme_kit::text;

/// Where a legacy runbook preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRunbookPreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyRunbookPreferenceStatus {
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

impl std::fmt::Display for LegacyRunbookPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy runbook preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookPreference {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub weight: f64,
    pub floor_value: f64,
    pub limit: i64,
    pub timezone: String,
    pub source: String,
    pub status: LegacyRunbookPreferenceStatus,
}

impl LegacyRunbookPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRunbookPreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRunbookPreferenceStatus) -> bool {
        match self.status {
            LegacyRunbookPreferenceStatus::Open => {
                match next {
                    LegacyRunbookPreferenceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRunbookPreferenceStatus::InReview => {
                return next == LegacyRunbookPreferenceStatus::Reopened
                    || next == LegacyRunbookPreferenceStatus::Archived;
            }
            LegacyRunbookPreferenceStatus::Resolved => {
                match next {
                    LegacyRunbookPreferenceStatus::Reopened | LegacyRunbookPreferenceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRunbookPreferenceStatus::Reopened => {
                return next == LegacyRunbookPreferenceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyRunbookPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy runbook preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.floor_value + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRunbookPreference {
        LegacyRunbookPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            weight: self.weight,
            floor_value: self.floor_value,
            limit: self.limit,
            timezone: self.timezone.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy runbook preference in status open.
pub fn new_legacy_runbook_preference(code: &str, label: &str) -> LegacyRunbookPreference {
    LegacyRunbookPreference {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        weight: 0.0,
        floor_value: 0.0,
        limit: 0,
        timezone: "".to_string(),
        source: "".to_string(),
        status: LegacyRunbookPreferenceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_runbook_preference("code-248", "label-309");
        assert_eq!(item.status, LegacyRunbookPreferenceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_legacy_runbook_preference("code-248", "label-309");
        if item.move_to(LegacyRunbookPreferenceStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRunbookPreferenceStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_runbook_preference("code-248", "label-309");
        item.status = LegacyRunbookPreferenceStatus::Archived;
        assert!(
            item.move_to(LegacyRunbookPreferenceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
