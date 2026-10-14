use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a tiered segment claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSegmentClaimStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredSegmentClaimStatus {
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

impl std::fmt::Display for TieredSegmentClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered segment claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentClaim {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub minimum: f64,
    pub pinned: bool,
    pub version: i64,
    pub locale: String,
    pub quantity: i64,
    pub tenant_id: String,
    pub burst: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: TieredSegmentClaimStatus,
}

impl TieredSegmentClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSegmentClaimStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSegmentClaimStatus) -> bool {
        match self.status {
            TieredSegmentClaimStatus::Open => {
                return next == TieredSegmentClaimStatus::Reopened || next == TieredSegmentClaimStatus::Archived;
            }
            TieredSegmentClaimStatus::InReview => {
                return next == TieredSegmentClaimStatus::Resolved;
            }
            TieredSegmentClaimStatus::Resolved => {
                return next == TieredSegmentClaimStatus::Reopened || next == TieredSegmentClaimStatus::Archived;
            }
            TieredSegmentClaimStatus::Reopened => {
                return next == TieredSegmentClaimStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredSegmentClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered segment claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered segment claim", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.burst as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredSegmentClaim {
        TieredSegmentClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            minimum: self.minimum,
            pinned: self.pinned,
            version: self.version,
            locale: self.locale.clone(),
            quantity: self.quantity,
            tenant_id: self.tenant_id.clone(),
            burst: self.burst,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tiered segment claim in status open.
pub fn new_tiered_segment_claim(code: &str, name: &str) -> TieredSegmentClaim {
    TieredSegmentClaim {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        minimum: 0.0,
        pinned: false,
        version: 0,
        locale: "".to_string(),
        quantity: 0,
        tenant_id: "".to_string(),
        burst: 0,
        attributes: BTreeMap::new(),
        status: TieredSegmentClaimStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_segment_claim("code-227", "name-510");
        assert_eq!(item.status, TieredSegmentClaimStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_tiered_segment_claim("code-227", "name-510");
        if item.move_to(TieredSegmentClaimStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredSegmentClaimStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_segment_claim("code-227", "name-510");
        item.status = TieredSegmentClaimStatus::Archived;
        assert!(
            item.move_to(TieredSegmentClaimStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
