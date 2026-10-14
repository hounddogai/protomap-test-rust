use super::*;

use acme_kit::text;

/// Where a fallback retention claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRetentionClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackRetentionClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for FallbackRetentionClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback retention claim as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionClaim {
    pub code: String,
    pub title: String,
    pub source: String,
    pub expires_at: i64,
    pub starts_at: i64,
    pub quantity: i64,
    pub tags: Vec<String>,
    pub retries: i64,
    pub unit: String,
    pub status: FallbackRetentionClaimStatus,
}

impl FallbackRetentionClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRetentionClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRetentionClaimStatus) -> bool {
        match self.status {
            FallbackRetentionClaimStatus::Draft => {
                match next {
                    FallbackRetentionClaimStatus::Pending | FallbackRetentionClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRetentionClaimStatus::Pending => {
                return next == FallbackRetentionClaimStatus::Closed;
            }
            FallbackRetentionClaimStatus::Active => {
                match next {
                    FallbackRetentionClaimStatus::Suspended | FallbackRetentionClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRetentionClaimStatus::Suspended => {
                return next == FallbackRetentionClaimStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackRetentionClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback retention claim {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.quantity as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRetentionClaim {
        FallbackRetentionClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            expires_at: self.expires_at,
            starts_at: self.starts_at,
            quantity: self.quantity,
            tags: self.tags.clone(),
            retries: self.retries,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a fallback retention claim in status draft.
pub fn new_fallback_retention_claim(code: &str, title: &str) -> FallbackRetentionClaim {
    FallbackRetentionClaim {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        expires_at: 0,
        starts_at: 0,
        quantity: 0,
        tags: Vec::new(),
        retries: 0,
        unit: "".to_string(),
        status: FallbackRetentionClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_retention_claim("code-539", "title-264");
        assert_eq!(item.status, FallbackRetentionClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_fallback_retention_claim("code-539", "title-264");
        if item.move_to(FallbackRetentionClaimStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRetentionClaimStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_retention_claim("code-539", "title-264");
        item.status = FallbackRetentionClaimStatus::Closed;
        assert!(
            item.move_to(FallbackRetentionClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
