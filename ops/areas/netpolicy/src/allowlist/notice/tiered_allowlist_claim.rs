use super::*;

use acme_kit::text;

/// Where a tiered allowlist claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAllowlistClaimStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredAllowlistClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredAllowlistClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered allowlist claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistClaim {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub updated_at: i64,
    pub pinned: bool,
    pub category: String,
    pub ceiling: f64,
    pub expires_at: i64,
    pub score: f64,
    pub country: String,
    pub status: TieredAllowlistClaimStatus,
}

impl TieredAllowlistClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAllowlistClaimStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAllowlistClaimStatus) -> bool {
        match self.status {
            TieredAllowlistClaimStatus::Requested => {
                return next == TieredAllowlistClaimStatus::Granted;
            }
            TieredAllowlistClaimStatus::Granted => {
                return next == TieredAllowlistClaimStatus::Revoked || next == TieredAllowlistClaimStatus::Expired;
            }
            TieredAllowlistClaimStatus::Revoked => {
                return next == TieredAllowlistClaimStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredAllowlistClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered allowlist claim {} cannot move from {} to {}",
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
        format!("{} {}", "tiered allowlist claim", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ceiling - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredAllowlistClaim {
        TieredAllowlistClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            updated_at: self.updated_at,
            pinned: self.pinned,
            category: self.category.clone(),
            ceiling: self.ceiling,
            expires_at: self.expires_at,
            score: self.score,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered allowlist claim in status requested.
pub fn new_tiered_allowlist_claim(code: &str, title: &str) -> TieredAllowlistClaim {
    TieredAllowlistClaim {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        updated_at: 0,
        pinned: false,
        category: "".to_string(),
        ceiling: 0.0,
        expires_at: 0,
        score: 0.0,
        country: "".to_string(),
        status: TieredAllowlistClaimStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tiered_allowlist_claim("code-214", "title-677");
        assert_eq!(item.status, TieredAllowlistClaimStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_tiered_allowlist_claim("code-214", "title-677");
        if item.move_to(TieredAllowlistClaimStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredAllowlistClaimStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tiered_allowlist_claim("code-214", "title-677");
        item.status = TieredAllowlistClaimStatus::Expired;
        assert!(
            item.move_to(TieredAllowlistClaimStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
