use super::*;

use acme_kit::text;

/// Where a release snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseSnapshotStatus {
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

impl std::fmt::Display for ReleaseSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSnapshot {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub namespace: String,
    pub maximum: f64,
    pub region: String,
    pub country: String,
    pub status: ReleaseSnapshotStatus,
}

impl ReleaseSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseSnapshotStatus) -> bool {
        match self.status {
            ReleaseSnapshotStatus::Open => {
                match next {
                    ReleaseSnapshotStatus::Reopened | ReleaseSnapshotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseSnapshotStatus::InReview => {
                match next {
                    ReleaseSnapshotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseSnapshotStatus::Resolved => {
                return next == ReleaseSnapshotStatus::Reopened;
            }
            ReleaseSnapshotStatus::Reopened => {
                match next {
                    ReleaseSnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: ReleaseSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "release snapshot", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.maximum * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseSnapshot {
        ReleaseSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            namespace: self.namespace.clone(),
            maximum: self.maximum,
            region: self.region.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a release snapshot in status open.
pub fn new_release_snapshot(code: &str, title: &str) -> ReleaseSnapshot {
    ReleaseSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        namespace: "".to_string(),
        maximum: 0.0,
        region: "".to_string(),
        country: "".to_string(),
        status: ReleaseSnapshotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_release_snapshot("code-897", "title-157");
        assert_eq!(item.status, ReleaseSnapshotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_release_snapshot("code-897", "title-157");
        if item.move_to(ReleaseSnapshotStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseSnapshotStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_release_snapshot("code-897", "title-157");
        item.status = ReleaseSnapshotStatus::Archived;
        assert!(
            item.move_to(ReleaseSnapshotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
