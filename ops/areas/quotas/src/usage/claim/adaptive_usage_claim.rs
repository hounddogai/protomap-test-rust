use super::*;

use acme_kit::text;

/// Where a adaptive usage claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveUsageClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveUsageClaimStatus {
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

impl std::fmt::Display for AdaptiveUsageClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive usage claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageClaim {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub cost: f64,
    pub segment: String,
    pub threshold: f64,
    pub fingerprint: String,
    pub revision: i64,
    pub created_at: i64,
    pub owner_id: String,
    pub checksum: String,
    pub status: AdaptiveUsageClaimStatus,
}

impl AdaptiveUsageClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveUsageClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveUsageClaimStatus) -> bool {
        match self.status {
            AdaptiveUsageClaimStatus::Draft => {
                match next {
                    AdaptiveUsageClaimStatus::Pending | AdaptiveUsageClaimStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveUsageClaimStatus::Pending => {
                match next {
                    AdaptiveUsageClaimStatus::Suspended | AdaptiveUsageClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveUsageClaimStatus::Active => {
                return next == AdaptiveUsageClaimStatus::Suspended || next == AdaptiveUsageClaimStatus::Closed;
            }
            AdaptiveUsageClaimStatus::Suspended => {
                match next {
                    AdaptiveUsageClaimStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptiveUsageClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive usage claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive usage claim", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.revision as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveUsageClaim {
        AdaptiveUsageClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            cost: self.cost,
            segment: self.segment.clone(),
            threshold: self.threshold,
            fingerprint: self.fingerprint.clone(),
            revision: self.revision,
            created_at: self.created_at,
            owner_id: self.owner_id.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a adaptive usage claim in status draft.
pub fn new_adaptive_usage_claim(code: &str, name: &str) -> AdaptiveUsageClaim {
    AdaptiveUsageClaim {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        cost: 0.0,
        segment: "".to_string(),
        threshold: 0.0,
        fingerprint: "".to_string(),
        revision: 0,
        created_at: 0,
        owner_id: "".to_string(),
        checksum: "".to_string(),
        status: AdaptiveUsageClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_usage_claim("code-378", "name-953");
        assert_eq!(item.status, AdaptiveUsageClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_adaptive_usage_claim("code-378", "name-953");
        if item.move_to(AdaptiveUsageClaimStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveUsageClaimStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_usage_claim("code-378", "name-953");
        item.status = AdaptiveUsageClaimStatus::Closed;
        assert!(
            item.move_to(AdaptiveUsageClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
