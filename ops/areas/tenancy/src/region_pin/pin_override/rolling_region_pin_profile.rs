use super::*;

use acme_kit::text;

/// Where a rolling region pin profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRegionPinProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingRegionPinProfileStatus {
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

impl std::fmt::Display for RollingRegionPinProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling region pin profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinProfile {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub ttl_seconds: i64,
    pub source: String,
    pub category: String,
    pub title: String,
    pub status: RollingRegionPinProfileStatus,
}

impl RollingRegionPinProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRegionPinProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRegionPinProfileStatus) -> bool {
        match self.status {
            RollingRegionPinProfileStatus::Open => {
                match next {
                    RollingRegionPinProfileStatus::Resolved | RollingRegionPinProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRegionPinProfileStatus::InReview => {
                return next == RollingRegionPinProfileStatus::Reopened
                    || next == RollingRegionPinProfileStatus::Archived;
            }
            RollingRegionPinProfileStatus::Resolved => {
                return next == RollingRegionPinProfileStatus::Reopened;
            }
            RollingRegionPinProfileStatus::Reopened => {
                return next == RollingRegionPinProfileStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingRegionPinProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling region pin profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling region pin profile", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRegionPinProfile {
        RollingRegionPinProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            ttl_seconds: self.ttl_seconds,
            source: self.source.clone(),
            category: self.category.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling region pin profile in status open.
pub fn new_rolling_region_pin_profile(code: &str, label: &str) -> RollingRegionPinProfile {
    RollingRegionPinProfile {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        ttl_seconds: 0,
        source: "".to_string(),
        category: "".to_string(),
        title: "".to_string(),
        status: RollingRegionPinProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_region_pin_profile("code-959", "label-762");
        assert_eq!(item.status, RollingRegionPinProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rolling_region_pin_profile("code-959", "label-762");
        if item.move_to(RollingRegionPinProfileStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRegionPinProfileStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_region_pin_profile("code-959", "label-762");
        item.status = RollingRegionPinProfileStatus::Archived;
        assert!(
            item.move_to(RollingRegionPinProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
