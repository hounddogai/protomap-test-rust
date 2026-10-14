use super::*;

use acme_kit::text;

/// Where a pending release digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingReleaseDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingReleaseDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PendingReleaseDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending release digest as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseDigest {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub owner_id: String,
    pub summary: String,
    pub checksum: String,
    pub key_id: String,
    pub status: PendingReleaseDigestStatus,
}

impl PendingReleaseDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingReleaseDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingReleaseDigestStatus) -> bool {
        match self.status {
            PendingReleaseDigestStatus::Planned => {
                match next {
                    PendingReleaseDigestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingReleaseDigestStatus::Scheduled => {
                match next {
                    PendingReleaseDigestStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingReleaseDigestStatus::InProgress => {
                return next == PendingReleaseDigestStatus::Cancelled;
            }
            PendingReleaseDigestStatus::Completed => {
                return next == PendingReleaseDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingReleaseDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending release digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> PendingReleaseDigest {
        PendingReleaseDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            owner_id: self.owner_id.clone(),
            summary: self.summary.clone(),
            checksum: self.checksum.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending release digest in status planned.
pub fn new_pending_release_digest(code: &str, title: &str) -> PendingReleaseDigest {
    PendingReleaseDigest {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        owner_id: "".to_string(),
        summary: "".to_string(),
        checksum: "".to_string(),
        key_id: "".to_string(),
        status: PendingReleaseDigestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_release_digest("code-159", "title-695");
        assert_eq!(item.status, PendingReleaseDigestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_pending_release_digest("code-159", "title-695");
        if item.move_to(PendingReleaseDigestStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingReleaseDigestStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_release_digest("code-159", "title-695");
        item.status = PendingReleaseDigestStatus::Cancelled;
        assert!(
            item.move_to(PendingReleaseDigestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
