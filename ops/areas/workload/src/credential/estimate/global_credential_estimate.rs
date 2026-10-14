use super::*;

use acme_kit::text;

/// Where a global credential estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalCredentialEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalCredentialEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalCredentialEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global credential estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCredentialEstimate {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub ttl_seconds: i64,
    pub segment: String,
    pub capacity: i64,
    pub priority: i64,
    pub tenant_id: String,
    pub key_id: String,
    pub threshold: f64,
    pub status: GlobalCredentialEstimateStatus,
}

impl GlobalCredentialEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalCredentialEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalCredentialEstimateStatus) -> bool {
        match self.status {
            GlobalCredentialEstimateStatus::Created => {
                return next == GlobalCredentialEstimateStatus::Retired;
            }
            GlobalCredentialEstimateStatus::Validated => {
                match next {
                    GlobalCredentialEstimateStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalCredentialEstimateStatus::Published => {
                return next == GlobalCredentialEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalCredentialEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global credential estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.capacity as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalCredentialEstimate {
        GlobalCredentialEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            ttl_seconds: self.ttl_seconds,
            segment: self.segment.clone(),
            capacity: self.capacity,
            priority: self.priority,
            tenant_id: self.tenant_id.clone(),
            key_id: self.key_id.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a global credential estimate in status created.
pub fn new_global_credential_estimate(code: &str, name: &str) -> GlobalCredentialEstimate {
    GlobalCredentialEstimate {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        ttl_seconds: 0,
        segment: "".to_string(),
        capacity: 0,
        priority: 0,
        tenant_id: "".to_string(),
        key_id: "".to_string(),
        threshold: 0.0,
        status: GlobalCredentialEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_credential_estimate("code-369", "name-795");
        assert_eq!(item.status, GlobalCredentialEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_global_credential_estimate("code-369", "name-795");
        if item.move_to(GlobalCredentialEstimateStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalCredentialEstimateStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_credential_estimate("code-369", "name-795");
        item.status = GlobalCredentialEstimateStatus::Retired;
        assert!(
            item.move_to(GlobalCredentialEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
