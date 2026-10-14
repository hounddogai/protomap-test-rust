use super::*;

use acme_kit::text;

/// Where a policy mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PolicyMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PolicyMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMapping {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub updated_at: i64,
    pub archived: bool,
    pub phone: String,
    pub quantity: i64,
    pub segment: String,
    pub cost: f64,
    pub ttl_seconds: i64,
    pub owner_id: String,
    pub status: PolicyMappingStatus,
}

impl PolicyMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyMappingStatus) -> bool {
        match self.status {
            PolicyMappingStatus::Requested => {
                return next == PolicyMappingStatus::Revoked || next == PolicyMappingStatus::Expired;
            }
            PolicyMappingStatus::Granted => {
                match next {
                    PolicyMappingStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyMappingStatus::Revoked => {
                match next {
                    PolicyMappingStatus::Expired => {
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

    pub fn move_to(&mut self, next: PolicyMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy mapping {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.updated_at as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyMapping {
        PolicyMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            updated_at: self.updated_at,
            archived: self.archived,
            phone: self.phone.clone(),
            quantity: self.quantity,
            segment: self.segment.clone(),
            cost: self.cost,
            ttl_seconds: self.ttl_seconds,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a policy mapping in status requested.
pub fn new_policy_mapping(code: &str, label: &str) -> PolicyMapping {
    PolicyMapping {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        updated_at: 0,
        archived: false,
        phone: "".to_string(),
        quantity: 0,
        segment: "".to_string(),
        cost: 0.0,
        ttl_seconds: 0,
        owner_id: "".to_string(),
        status: PolicyMappingStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_policy_mapping("code-857", "label-870");
        assert_eq!(item.status, PolicyMappingStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_policy_mapping("code-857", "label-870");
        if item.move_to(PolicyMappingStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyMappingStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_policy_mapping("code-857", "label-870");
        item.status = PolicyMappingStatus::Expired;
        assert!(
            item.move_to(PolicyMappingStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
