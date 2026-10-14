use super::*;

use acme_kit::text;

/// Where a fallback release snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackReleaseSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackReleaseSnapshotStatus {
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

impl std::fmt::Display for FallbackReleaseSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback release snapshot as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseSnapshot {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub revision: i64,
    pub version: i64,
    pub ttl_seconds: i64,
    pub channel: String,
    pub fingerprint: String,
    pub sequence: i64,
    pub status: FallbackReleaseSnapshotStatus,
}

impl FallbackReleaseSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackReleaseSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackReleaseSnapshotStatus) -> bool {
        match self.status {
            FallbackReleaseSnapshotStatus::Open => {
                return next == FallbackReleaseSnapshotStatus::Reopened;
            }
            FallbackReleaseSnapshotStatus::InReview => {
                match next {
                    FallbackReleaseSnapshotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReleaseSnapshotStatus::Resolved => {
                return next == FallbackReleaseSnapshotStatus::Reopened;
            }
            FallbackReleaseSnapshotStatus::Reopened => {
                match next {
                    FallbackReleaseSnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: FallbackReleaseSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback release snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.revision as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackReleaseSnapshot {
        FallbackReleaseSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            revision: self.revision,
            version: self.version,
            ttl_seconds: self.ttl_seconds,
            channel: self.channel.clone(),
            fingerprint: self.fingerprint.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback release snapshot in status open.
pub fn new_fallback_release_snapshot(code: &str, title: &str) -> FallbackReleaseSnapshot {
    FallbackReleaseSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        revision: 0,
        version: 0,
        ttl_seconds: 0,
        channel: "".to_string(),
        fingerprint: "".to_string(),
        sequence: 0,
        status: FallbackReleaseSnapshotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_release_snapshot("code-334", "title-984");
        assert_eq!(item.status, FallbackReleaseSnapshotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_fallback_release_snapshot("code-334", "title-984");
        if item.move_to(FallbackReleaseSnapshotStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackReleaseSnapshotStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_release_snapshot("code-334", "title-984");
        item.status = FallbackReleaseSnapshotStatus::Archived;
        assert!(
            item.move_to(FallbackReleaseSnapshotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
