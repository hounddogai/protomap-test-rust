use super::*;

use acme_kit::text;

/// Where a primary identity claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryIdentityClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryIdentityClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PrimaryIdentityClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary identity claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityClaim {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub quota_units: i64,
    pub name: String,
    pub country: String,
    pub expires_at: i64,
    pub status: PrimaryIdentityClaimStatus,
}

impl PrimaryIdentityClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryIdentityClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryIdentityClaimStatus) -> bool {
        match self.status {
            PrimaryIdentityClaimStatus::Planned => {
                return next == PrimaryIdentityClaimStatus::Scheduled || next == PrimaryIdentityClaimStatus::Completed;
            }
            PrimaryIdentityClaimStatus::Scheduled => {
                return next == PrimaryIdentityClaimStatus::Cancelled;
            }
            PrimaryIdentityClaimStatus::InProgress => {
                return next == PrimaryIdentityClaimStatus::Cancelled;
            }
            PrimaryIdentityClaimStatus::Completed => {
                match next {
                    PrimaryIdentityClaimStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimaryIdentityClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary identity claim {} cannot move from {} to {}",
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
        format!("{} {}", "primary identity claim", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.expires_at as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryIdentityClaim {
        PrimaryIdentityClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            quota_units: self.quota_units,
            name: self.name.clone(),
            country: self.country.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary identity claim in status planned.
pub fn new_primary_identity_claim(code: &str, title: &str) -> PrimaryIdentityClaim {
    PrimaryIdentityClaim {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        quota_units: 0,
        name: "".to_string(),
        country: "".to_string(),
        expires_at: 0,
        status: PrimaryIdentityClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_identity_claim("code-751", "title-341");
        assert_eq!(item.status, PrimaryIdentityClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_primary_identity_claim("code-751", "title-341");
        if item.move_to(PrimaryIdentityClaimStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryIdentityClaimStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_identity_claim("code-751", "title-341");
        item.status = PrimaryIdentityClaimStatus::Cancelled;
        assert!(
            item.move_to(PrimaryIdentityClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
