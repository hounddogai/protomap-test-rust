use super::*;

use acme_kit::text;

/// Where a regional peering digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPeeringDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalPeeringDigestStatus {
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

impl std::fmt::Display for RegionalPeeringDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional peering digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringDigest {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub created_at: i64,
    pub expires_at: i64,
    pub external_ref: String,
    pub cipher: String,
    pub status: RegionalPeeringDigestStatus,
}

impl RegionalPeeringDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPeeringDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPeeringDigestStatus) -> bool {
        match self.status {
            RegionalPeeringDigestStatus::Open => {
                return next == RegionalPeeringDigestStatus::Archived;
            }
            RegionalPeeringDigestStatus::InReview => {
                match next {
                    RegionalPeeringDigestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPeeringDigestStatus::Resolved => {
                return next == RegionalPeeringDigestStatus::Archived;
            }
            RegionalPeeringDigestStatus::Reopened => {
                return next == RegionalPeeringDigestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalPeeringDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional peering digest {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "regional peering digest", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.retries as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPeeringDigest {
        RegionalPeeringDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            created_at: self.created_at,
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional peering digest in status open.
pub fn new_regional_peering_digest(code: &str, name: &str) -> RegionalPeeringDigest {
    RegionalPeeringDigest {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        created_at: 0,
        expires_at: 0,
        external_ref: "".to_string(),
        cipher: "".to_string(),
        status: RegionalPeeringDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_peering_digest("code-240", "name-353");
        assert_eq!(item.status, RegionalPeeringDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_regional_peering_digest("code-240", "name-353");
        if item.move_to(RegionalPeeringDigestStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalPeeringDigestStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_peering_digest("code-240", "name-353");
        item.status = RegionalPeeringDigestStatus::Archived;
        assert!(
            item.move_to(RegionalPeeringDigestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
