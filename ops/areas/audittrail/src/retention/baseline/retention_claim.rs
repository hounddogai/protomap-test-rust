use super::*;

use acme_kit::text;

/// Where a retention claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RetentionClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RetentionClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionClaim {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub floor_value: f64,
    pub name: String,
    pub rotation_days: i64,
    pub currency: String,
    pub expires_at: i64,
    pub summary: String,
    pub quota_units: i64,
    pub status: RetentionClaimStatus,
}

impl RetentionClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionClaimStatus) -> bool {
        match self.status {
            RetentionClaimStatus::Proposed => {
                match next {
                    RetentionClaimStatus::Approved | RetentionClaimStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionClaimStatus::Approved => {
                match next {
                    RetentionClaimStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionClaimStatus::Rejected => {
                return next == RetentionClaimStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention claim {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        (self.floor_value * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionClaim {
        RetentionClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            floor_value: self.floor_value,
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            currency: self.currency.clone(),
            expires_at: self.expires_at,
            summary: self.summary.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a retention claim in status proposed.
pub fn new_retention_claim(code: &str, title: &str) -> RetentionClaim {
    RetentionClaim {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        floor_value: 0.0,
        name: "".to_string(),
        rotation_days: 0,
        currency: "".to_string(),
        expires_at: 0,
        summary: "".to_string(),
        quota_units: 0,
        status: RetentionClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_retention_claim("code-338", "title-994");
        assert_eq!(item.status, RetentionClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_retention_claim("code-338", "title-994");
        if item.move_to(RetentionClaimStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionClaimStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_retention_claim("code-338", "title-994");
        item.status = RetentionClaimStatus::Withdrawn;
        assert!(
            item.move_to(RetentionClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
