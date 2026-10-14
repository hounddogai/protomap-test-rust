use super::*;

use acme_kit::text;

/// Where a seasonal limit digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalLimitDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalLimitDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalLimitDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal limit digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitDigest {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub checksum: String,
    pub maximum: f64,
    pub status: SeasonalLimitDigestStatus,
}

impl SeasonalLimitDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalLimitDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalLimitDigestStatus) -> bool {
        match self.status {
            SeasonalLimitDigestStatus::Requested => {
                return next == SeasonalLimitDigestStatus::Granted || next == SeasonalLimitDigestStatus::Revoked;
            }
            SeasonalLimitDigestStatus::Granted => {
                match next {
                    SeasonalLimitDigestStatus::Revoked | SeasonalLimitDigestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalLimitDigestStatus::Revoked => {
                return next == SeasonalLimitDigestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalLimitDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal limit digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> SeasonalLimitDigest {
        SeasonalLimitDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            checksum: self.checksum.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal limit digest in status requested.
pub fn new_seasonal_limit_digest(code: &str, title: &str) -> SeasonalLimitDigest {
    SeasonalLimitDigest {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        checksum: "".to_string(),
        maximum: 0.0,
        status: SeasonalLimitDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_limit_digest("code-317", "title-686");
        assert_eq!(item.status, SeasonalLimitDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_limit_digest("code-317", "title-686");
        if item.move_to(SeasonalLimitDigestStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalLimitDigestStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_limit_digest("code-317", "title-686");
        item.status = SeasonalLimitDigestStatus::Expired;
        assert!(
            item.move_to(SeasonalLimitDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
