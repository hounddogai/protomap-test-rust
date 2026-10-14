use super::*;

use acme_kit::text;

/// Where a seasonal release digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalReleaseDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalReleaseDigestStatus {
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

impl std::fmt::Display for SeasonalReleaseDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal release digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseDigest {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub owner_id: String,
    pub source: String,
    pub ttl_seconds: i64,
    pub title: String,
    pub currency: String,
    pub status: SeasonalReleaseDigestStatus,
}

impl SeasonalReleaseDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalReleaseDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalReleaseDigestStatus) -> bool {
        match self.status {
            SeasonalReleaseDigestStatus::New => {
                match next {
                    SeasonalReleaseDigestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalReleaseDigestStatus::Queued => {
                return next == SeasonalReleaseDigestStatus::Done;
            }
            SeasonalReleaseDigestStatus::Running => {
                return next == SeasonalReleaseDigestStatus::Failed;
            }
            SeasonalReleaseDigestStatus::Done => {
                match next {
                    SeasonalReleaseDigestStatus::Failed => {
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

    pub fn move_to(&mut self, next: SeasonalReleaseDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal release digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal release digest", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ends_at as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalReleaseDigest {
        SeasonalReleaseDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            ttl_seconds: self.ttl_seconds,
            title: self.title.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a seasonal release digest in status new.
pub fn new_seasonal_release_digest(code: &str, name: &str) -> SeasonalReleaseDigest {
    SeasonalReleaseDigest {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        owner_id: "".to_string(),
        source: "".to_string(),
        ttl_seconds: 0,
        title: "".to_string(),
        currency: "".to_string(),
        status: SeasonalReleaseDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_release_digest("code-572", "name-895");
        assert_eq!(item.status, SeasonalReleaseDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_seasonal_release_digest("code-572", "name-895");
        if item.move_to(SeasonalReleaseDigestStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalReleaseDigestStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_release_digest("code-572", "name-895");
        item.status = SeasonalReleaseDigestStatus::Failed;
        assert!(
            item.move_to(SeasonalReleaseDigestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
