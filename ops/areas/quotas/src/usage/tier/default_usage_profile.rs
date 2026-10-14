use super::*;

use acme_kit::text;

/// Where a default usage profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultUsageProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultUsageProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultUsageProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default usage profile as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageProfile {
    pub code: String,
    pub name: String,
    pub label: String,
    pub cluster: String,
    pub tenant_id: String,
    pub created_at: i64,
    pub namespace: String,
    pub phone: String,
    pub description: String,
    pub status: DefaultUsageProfileStatus,
}

impl DefaultUsageProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultUsageProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultUsageProfileStatus) -> bool {
        match self.status {
            DefaultUsageProfileStatus::Requested => {
                match next {
                    DefaultUsageProfileStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultUsageProfileStatus::Granted => {
                return next == DefaultUsageProfileStatus::Revoked;
            }
            DefaultUsageProfileStatus::Revoked => {
                return next == DefaultUsageProfileStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultUsageProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default usage profile {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> DefaultUsageProfile {
        DefaultUsageProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            tenant_id: self.tenant_id.clone(),
            created_at: self.created_at,
            namespace: self.namespace.clone(),
            phone: self.phone.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a default usage profile in status requested.
pub fn new_default_usage_profile(code: &str, name: &str) -> DefaultUsageProfile {
    DefaultUsageProfile {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        cluster: "".to_string(),
        tenant_id: "".to_string(),
        created_at: 0,
        namespace: "".to_string(),
        phone: "".to_string(),
        description: "".to_string(),
        status: DefaultUsageProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_usage_profile("code-332", "name-762");
        assert_eq!(item.status, DefaultUsageProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_default_usage_profile("code-332", "name-762");
        if item.move_to(DefaultUsageProfileStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultUsageProfileStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_usage_profile("code-332", "name-762");
        item.status = DefaultUsageProfileStatus::Expired;
        assert!(
            item.move_to(DefaultUsageProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
