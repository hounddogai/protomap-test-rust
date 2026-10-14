use super::*;

use acme_kit::text;

/// Where a primary usage override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryUsageOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryUsageOverrideStatus {
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

impl std::fmt::Display for PrimaryUsageOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary usage override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageOverride {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub category: String,
    pub capacity: i64,
    pub archived: bool,
    pub unit: String,
    pub ttl_seconds: i64,
    pub visible: bool,
    pub label: String,
    pub quota_units: i64,
    pub status: PrimaryUsageOverrideStatus,
}

impl PrimaryUsageOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryUsageOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryUsageOverrideStatus) -> bool {
        match self.status {
            PrimaryUsageOverrideStatus::Open => {
                match next {
                    PrimaryUsageOverrideStatus::Reopened | PrimaryUsageOverrideStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryUsageOverrideStatus::InReview => {
                return next == PrimaryUsageOverrideStatus::Resolved || next == PrimaryUsageOverrideStatus::Archived;
            }
            PrimaryUsageOverrideStatus::Resolved => {
                match next {
                    PrimaryUsageOverrideStatus::Reopened | PrimaryUsageOverrideStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryUsageOverrideStatus::Reopened => {
                return next == PrimaryUsageOverrideStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryUsageOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary usage override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary usage override", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quota_units as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryUsageOverride {
        PrimaryUsageOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            category: self.category.clone(),
            capacity: self.capacity,
            archived: self.archived,
            unit: self.unit.clone(),
            ttl_seconds: self.ttl_seconds,
            visible: self.visible,
            label: self.label.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a primary usage override in status open.
pub fn new_primary_usage_override(code: &str, title: &str) -> PrimaryUsageOverride {
    PrimaryUsageOverride {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        category: "".to_string(),
        capacity: 0,
        archived: false,
        unit: "".to_string(),
        ttl_seconds: 0,
        visible: false,
        label: "".to_string(),
        quota_units: 0,
        status: PrimaryUsageOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_usage_override("code-797", "title-201");
        assert_eq!(item.status, PrimaryUsageOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_primary_usage_override("code-797", "title-201");
        if item.move_to(PrimaryUsageOverrideStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryUsageOverrideStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_usage_override("code-797", "title-201");
        item.status = PrimaryUsageOverrideStatus::Archived;
        assert!(
            item.move_to(PrimaryUsageOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
