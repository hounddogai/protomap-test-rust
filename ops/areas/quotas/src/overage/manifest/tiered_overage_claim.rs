use super::*;

use acme_kit::text;

/// Where a tiered overage claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredOverageClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredOverageClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredOverageClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered overage claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageClaim {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub priority: i64,
    pub description: String,
    pub visible: bool,
    pub attempts: i64,
    pub timezone: String,
    pub cluster: String,
    pub host: String,
    pub updated_at: i64,
    pub status: TieredOverageClaimStatus,
}

impl TieredOverageClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredOverageClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredOverageClaimStatus) -> bool {
        match self.status {
            TieredOverageClaimStatus::Proposed => {
                match next {
                    TieredOverageClaimStatus::Approved | TieredOverageClaimStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageClaimStatus::Approved => {
                match next {
                    TieredOverageClaimStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageClaimStatus::Rejected => {
                return next == TieredOverageClaimStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredOverageClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered overage claim {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.discount - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredOverageClaim {
        TieredOverageClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            priority: self.priority,
            description: self.description.clone(),
            visible: self.visible,
            attempts: self.attempts,
            timezone: self.timezone.clone(),
            cluster: self.cluster.clone(),
            host: self.host.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tiered overage claim in status proposed.
pub fn new_tiered_overage_claim(code: &str, title: &str) -> TieredOverageClaim {
    TieredOverageClaim {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        priority: 0,
        description: "".to_string(),
        visible: false,
        attempts: 0,
        timezone: "".to_string(),
        cluster: "".to_string(),
        host: "".to_string(),
        updated_at: 0,
        status: TieredOverageClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_overage_claim("code-902", "title-882");
        assert_eq!(item.status, TieredOverageClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tiered_overage_claim("code-902", "title-882");
        if item.move_to(TieredOverageClaimStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredOverageClaimStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_overage_claim("code-902", "title-882");
        item.status = TieredOverageClaimStatus::Withdrawn;
        assert!(
            item.move_to(TieredOverageClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
