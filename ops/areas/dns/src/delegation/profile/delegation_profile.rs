use super::*;

use acme_kit::text;

/// Where a delegation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation profile as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationProfile {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub checksum: String,
    pub contact_email: String,
    pub expires_at: i64,
    pub country: String,
    pub status: DelegationProfileStatus,
}

impl DelegationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationProfileStatus) -> bool {
        match self.status {
            DelegationProfileStatus::Proposed => {
                match next {
                    DelegationProfileStatus::Approved | DelegationProfileStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationProfileStatus::Approved => {
                match next {
                    DelegationProfileStatus::Rejected | DelegationProfileStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationProfileStatus::Rejected => {
                match next {
                    DelegationProfileStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DelegationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation profile {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ends_at as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationProfile {
        DelegationProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            checksum: self.checksum.clone(),
            contact_email: self.contact_email.clone(),
            expires_at: self.expires_at,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a delegation profile in status proposed.
pub fn new_delegation_profile(code: &str, name: &str) -> DelegationProfile {
    DelegationProfile {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        checksum: "".to_string(),
        contact_email: "".to_string(),
        expires_at: 0,
        country: "".to_string(),
        status: DelegationProfileStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_delegation_profile("code-316", "name-65");
        assert_eq!(item.status, DelegationProfileStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_delegation_profile("code-316", "name-65");
        if item.move_to(DelegationProfileStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationProfileStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_delegation_profile("code-316", "name-65");
        item.status = DelegationProfileStatus::Withdrawn;
        assert!(
            item.move_to(DelegationProfileStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
