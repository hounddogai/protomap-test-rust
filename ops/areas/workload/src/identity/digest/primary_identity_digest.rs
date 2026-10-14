use super::*;

use acme_kit::text;

/// Where a primary identity digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryIdentityDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryIdentityDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PrimaryIdentityDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary identity digest as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityDigest {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub updated_at: i64,
    pub ends_at: i64,
    pub minimum: f64,
    pub status: PrimaryIdentityDigestStatus,
}

impl PrimaryIdentityDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryIdentityDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryIdentityDigestStatus) -> bool {
        match self.status {
            PrimaryIdentityDigestStatus::New => {
                match next {
                    PrimaryIdentityDigestStatus::Done | PrimaryIdentityDigestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryIdentityDigestStatus::Queued => {
                return next == PrimaryIdentityDigestStatus::Running;
            }
            PrimaryIdentityDigestStatus::Running => {
                return next == PrimaryIdentityDigestStatus::Done || next == PrimaryIdentityDigestStatus::Failed;
            }
            PrimaryIdentityDigestStatus::Done => {
                match next {
                    PrimaryIdentityDigestStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimaryIdentityDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary identity digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryIdentityDigest {
        PrimaryIdentityDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            updated_at: self.updated_at,
            ends_at: self.ends_at,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary identity digest in status new.
pub fn new_primary_identity_digest(code: &str, title: &str) -> PrimaryIdentityDigest {
    PrimaryIdentityDigest {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        updated_at: 0,
        ends_at: 0,
        minimum: 0.0,
        status: PrimaryIdentityDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_identity_digest("code-551", "title-751");
        assert_eq!(item.status, PrimaryIdentityDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_primary_identity_digest("code-551", "title-751");
        if item.move_to(PrimaryIdentityDigestStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryIdentityDigestStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_identity_digest("code-551", "title-751");
        item.status = PrimaryIdentityDigestStatus::Failed;
        assert!(
            item.move_to(PrimaryIdentityDigestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
