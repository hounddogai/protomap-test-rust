use super::*;

use acme_kit::text;

/// Where a rolling usage digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingUsageDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingUsageDigestStatus {
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

impl std::fmt::Display for RollingUsageDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling usage digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageDigest {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub burst: i64,
    pub segment: String,
    pub status: RollingUsageDigestStatus,
}

impl RollingUsageDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingUsageDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingUsageDigestStatus) -> bool {
        match self.status {
            RollingUsageDigestStatus::New => {
                match next {
                    RollingUsageDigestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingUsageDigestStatus::Queued => {
                match next {
                    RollingUsageDigestStatus::Running | RollingUsageDigestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingUsageDigestStatus::Running => {
                return next == RollingUsageDigestStatus::Done;
            }
            RollingUsageDigestStatus::Done => {
                return next == RollingUsageDigestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingUsageDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling usage digest {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.burst as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingUsageDigest {
        RollingUsageDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            burst: self.burst,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rolling usage digest in status new.
pub fn new_rolling_usage_digest(code: &str, title: &str) -> RollingUsageDigest {
    RollingUsageDigest {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        burst: 0,
        segment: "".to_string(),
        status: RollingUsageDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_usage_digest("code-197", "title-825");
        assert_eq!(item.status, RollingUsageDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_rolling_usage_digest("code-197", "title-825");
        if item.move_to(RollingUsageDigestStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingUsageDigestStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_usage_digest("code-197", "title-825");
        item.status = RollingUsageDigestStatus::Failed;
        assert!(
            item.move_to(RollingUsageDigestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
