use super::*;

use acme_kit::text;

/// Where a timeline manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TimelineManifestStatus {
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

impl std::fmt::Display for TimelineManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One timeline manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineManifest {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub unit: String,
    pub locale: String,
    pub discount: f64,
    pub maximum: f64,
    pub archived: bool,
    pub status: TimelineManifestStatus,
}

impl TimelineManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineManifestStatus) -> bool {
        match self.status {
            TimelineManifestStatus::Open => {
                return next == TimelineManifestStatus::InReview;
            }
            TimelineManifestStatus::InReview => {
                match next {
                    TimelineManifestStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineManifestStatus::Resolved => {
                return next == TimelineManifestStatus::Reopened || next == TimelineManifestStatus::Archived;
            }
            TimelineManifestStatus::Reopened => {
                match next {
                    TimelineManifestStatus::Archived => {
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

    pub fn move_to(&mut self, next: TimelineManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline manifest {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.created_at as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineManifest {
        TimelineManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            unit: self.unit.clone(),
            locale: self.locale.clone(),
            discount: self.discount,
            maximum: self.maximum,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a timeline manifest in status open.
pub fn new_timeline_manifest(code: &str, label: &str) -> TimelineManifest {
    TimelineManifest {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        unit: "".to_string(),
        locale: "".to_string(),
        discount: 0.0,
        maximum: 0.0,
        archived: false,
        status: TimelineManifestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_timeline_manifest("code-862", "label-504");
        assert_eq!(item.status, TimelineManifestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_timeline_manifest("code-862", "label-504");
        if item.move_to(TimelineManifestStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineManifestStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_timeline_manifest("code-862", "label-504");
        item.status = TimelineManifestStatus::Archived;
        assert!(
            item.move_to(TimelineManifestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
