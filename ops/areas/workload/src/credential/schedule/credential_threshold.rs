use super::*;

use acme_kit::text;

/// Where a credential threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialThresholdStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CredentialThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CredentialThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialThreshold {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub notes: String,
    pub capacity: i64,
    pub tenant_id: String,
    pub status: CredentialThresholdStatus,
}

impl CredentialThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialThresholdStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialThresholdStatus) -> bool {
        match self.status {
            CredentialThresholdStatus::Created => {
                return next == CredentialThresholdStatus::Validated || next == CredentialThresholdStatus::Retired;
            }
            CredentialThresholdStatus::Validated => {
                match next {
                    CredentialThresholdStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialThresholdStatus::Published => {
                match next {
                    CredentialThresholdStatus::Retired => {
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

    pub fn move_to(&mut self, next: CredentialThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> CredentialThreshold {
        CredentialThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            notes: self.notes.clone(),
            capacity: self.capacity,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a credential threshold in status created.
pub fn new_credential_threshold(code: &str, title: &str) -> CredentialThreshold {
    CredentialThreshold {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        notes: "".to_string(),
        capacity: 0,
        tenant_id: "".to_string(),
        status: CredentialThresholdStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_credential_threshold("code-342", "title-956");
        assert_eq!(item.status, CredentialThresholdStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_credential_threshold("code-342", "title-956");
        if item.move_to(CredentialThresholdStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialThresholdStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_credential_threshold("code-342", "title-956");
        item.status = CredentialThresholdStatus::Retired;
        assert!(
            item.move_to(CredentialThresholdStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
