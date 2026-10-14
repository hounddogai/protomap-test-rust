use super::*;

use acme_kit::text;

/// Where a tiered nonce target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredNonceTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredNonceTargetStatus {
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

impl std::fmt::Display for TieredNonceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered nonce target as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceTarget {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub weight: f64,
    pub attempts: i64,
    pub fingerprint: String,
    pub tenant_id: String,
    pub pinned: bool,
    pub ends_at: i64,
    pub name: String,
    pub status: TieredNonceTargetStatus,
}

impl TieredNonceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredNonceTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredNonceTargetStatus) -> bool {
        match self.status {
            TieredNonceTargetStatus::Open => {
                match next {
                    TieredNonceTargetStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredNonceTargetStatus::InReview => {
                match next {
                    TieredNonceTargetStatus::Resolved | TieredNonceTargetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredNonceTargetStatus::Resolved => {
                return next == TieredNonceTargetStatus::Reopened || next == TieredNonceTargetStatus::Archived;
            }
            TieredNonceTargetStatus::Reopened => {
                return next == TieredNonceTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredNonceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered nonce target {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.weight * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredNonceTarget {
        TieredNonceTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            weight: self.weight,
            attempts: self.attempts,
            fingerprint: self.fingerprint.clone(),
            tenant_id: self.tenant_id.clone(),
            pinned: self.pinned,
            ends_at: self.ends_at,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered nonce target in status open.
pub fn new_tiered_nonce_target(code: &str, title: &str) -> TieredNonceTarget {
    TieredNonceTarget {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        weight: 0.0,
        attempts: 0,
        fingerprint: "".to_string(),
        tenant_id: "".to_string(),
        pinned: false,
        ends_at: 0,
        name: "".to_string(),
        status: TieredNonceTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_nonce_target("code-377", "title-384");
        assert_eq!(item.status, TieredNonceTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_tiered_nonce_target("code-377", "title-384");
        if item.move_to(TieredNonceTargetStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredNonceTargetStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_nonce_target("code-377", "title-384");
        item.status = TieredNonceTargetStatus::Archived;
        assert!(
            item.move_to(TieredNonceTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
