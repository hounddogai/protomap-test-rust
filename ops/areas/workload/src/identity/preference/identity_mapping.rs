use super::*;

use acme_kit::text;

/// Where a identity mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IdentityMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IdentityMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMapping {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub ratio: f64,
    pub source: String,
    pub quota_units: i64,
    pub quantity: i64,
    pub created_at: i64,
    pub principal: String,
    pub status: IdentityMappingStatus,
}

impl IdentityMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityMappingStatus) -> bool {
        match self.status {
            IdentityMappingStatus::Requested => {
                match next {
                    IdentityMappingStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityMappingStatus::Granted => {
                return next == IdentityMappingStatus::Expired;
            }
            IdentityMappingStatus::Revoked => {
                match next {
                    IdentityMappingStatus::Expired => {
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

    pub fn move_to(&mut self, next: IdentityMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity mapping {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quantity as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityMapping {
        IdentityMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            ratio: self.ratio,
            source: self.source.clone(),
            quota_units: self.quota_units,
            quantity: self.quantity,
            created_at: self.created_at,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a identity mapping in status requested.
pub fn new_identity_mapping(code: &str, title: &str) -> IdentityMapping {
    IdentityMapping {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        ratio: 0.0,
        source: "".to_string(),
        quota_units: 0,
        quantity: 0,
        created_at: 0,
        principal: "".to_string(),
        status: IdentityMappingStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_identity_mapping("code-526", "title-473");
        assert_eq!(item.status, IdentityMappingStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_identity_mapping("code-526", "title-473");
        if item.move_to(IdentityMappingStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityMappingStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_identity_mapping("code-526", "title-473");
        item.status = IdentityMappingStatus::Expired;
        assert!(
            item.move_to(IdentityMappingStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
