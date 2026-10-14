use super::*;

use acme_kit::text;

/// Where a release baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseBaselineStatus {
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

impl std::fmt::Display for ReleaseBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One release baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBaseline {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub notes: String,
    pub tenant_id: String,
    pub floor_value: f64,
    pub discount: f64,
    pub window_days: i64,
    pub audience: String,
    pub external_ref: String,
    pub status: ReleaseBaselineStatus,
}

impl ReleaseBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseBaselineStatus) -> bool {
        match self.status {
            ReleaseBaselineStatus::Open => {
                match next {
                    ReleaseBaselineStatus::Resolved | ReleaseBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseBaselineStatus::InReview => {
                return next == ReleaseBaselineStatus::Reopened;
            }
            ReleaseBaselineStatus::Resolved => {
                match next {
                    ReleaseBaselineStatus::Reopened | ReleaseBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseBaselineStatus::Reopened => {
                return next == ReleaseBaselineStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.window_days as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseBaseline {
        ReleaseBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            notes: self.notes.clone(),
            tenant_id: self.tenant_id.clone(),
            floor_value: self.floor_value,
            discount: self.discount,
            window_days: self.window_days,
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a release baseline in status open.
pub fn new_release_baseline(code: &str, label: &str) -> ReleaseBaseline {
    ReleaseBaseline {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        notes: "".to_string(),
        tenant_id: "".to_string(),
        floor_value: 0.0,
        discount: 0.0,
        window_days: 0,
        audience: "".to_string(),
        external_ref: "".to_string(),
        status: ReleaseBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_release_baseline("code-857", "label-91");
        assert_eq!(item.status, ReleaseBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_release_baseline("code-857", "label-91");
        if item.move_to(ReleaseBaselineStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseBaselineStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_release_baseline("code-857", "label-91");
        item.status = ReleaseBaselineStatus::Archived;
        assert!(
            item.move_to(ReleaseBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
