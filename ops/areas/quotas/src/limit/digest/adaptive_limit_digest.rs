use super::*;

use acme_kit::text;

/// Where a adaptive limit digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveLimitDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveLimitDigestStatus {
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

impl std::fmt::Display for AdaptiveLimitDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive limit digest as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitDigest {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub external_ref: String,
    pub description: String,
    pub cluster: String,
    pub name: String,
    pub batch_size: i64,
    pub checksum: String,
    pub status: AdaptiveLimitDigestStatus,
}

impl AdaptiveLimitDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveLimitDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveLimitDigestStatus) -> bool {
        match self.status {
            AdaptiveLimitDigestStatus::New => {
                match next {
                    AdaptiveLimitDigestStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveLimitDigestStatus::Queued => {
                match next {
                    AdaptiveLimitDigestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveLimitDigestStatus::Running => {
                match next {
                    AdaptiveLimitDigestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveLimitDigestStatus::Done => {
                return next == AdaptiveLimitDigestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveLimitDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive limit digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        (self.ratio * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveLimitDigest {
        AdaptiveLimitDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            external_ref: self.external_ref.clone(),
            description: self.description.clone(),
            cluster: self.cluster.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a adaptive limit digest in status new.
pub fn new_adaptive_limit_digest(code: &str, title: &str) -> AdaptiveLimitDigest {
    AdaptiveLimitDigest {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        external_ref: "".to_string(),
        description: "".to_string(),
        cluster: "".to_string(),
        name: "".to_string(),
        batch_size: 0,
        checksum: "".to_string(),
        status: AdaptiveLimitDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_limit_digest("code-85", "title-321");
        assert_eq!(item.status, AdaptiveLimitDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_adaptive_limit_digest("code-85", "title-321");
        if item.move_to(AdaptiveLimitDigestStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveLimitDigestStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_limit_digest("code-85", "title-321");
        item.status = AdaptiveLimitDigestStatus::Failed;
        assert!(
            item.move_to(AdaptiveLimitDigestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
