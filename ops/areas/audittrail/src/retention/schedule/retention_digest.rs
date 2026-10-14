use super::*;

use acme_kit::text;

/// Where a retention digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionDigestStatus {
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

impl std::fmt::Display for RetentionDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention digest as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDigest {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub reviewer: String,
    pub tags: Vec<String>,
    pub expires_at: i64,
    pub window_days: i64,
    pub status: RetentionDigestStatus,
}

impl RetentionDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionDigestStatus) -> bool {
        match self.status {
            RetentionDigestStatus::New => {
                return next == RetentionDigestStatus::Queued;
            }
            RetentionDigestStatus::Queued => {
                return next == RetentionDigestStatus::Done;
            }
            RetentionDigestStatus::Running => {
                match next {
                    RetentionDigestStatus::Done | RetentionDigestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionDigestStatus::Done => {
                return next == RetentionDigestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.expires_at as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionDigest {
        RetentionDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            reviewer: self.reviewer.clone(),
            tags: self.tags.clone(),
            expires_at: self.expires_at,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention digest in status new.
pub fn new_retention_digest(code: &str, label: &str) -> RetentionDigest {
    RetentionDigest {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        reviewer: "".to_string(),
        tags: Vec::new(),
        expires_at: 0,
        window_days: 0,
        status: RetentionDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_retention_digest("code-497", "label-80");
        assert_eq!(item.status, RetentionDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_retention_digest("code-497", "label-80");
        if item.move_to(RetentionDigestStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionDigestStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_retention_digest("code-497", "label-80");
        item.status = RetentionDigestStatus::Failed;
        assert!(
            item.move_to(RetentionDigestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
