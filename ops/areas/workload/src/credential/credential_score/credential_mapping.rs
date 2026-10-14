use super::*;

use acme_kit::text;

/// Where a credential mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialMappingStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CredentialMappingStatus {
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

impl std::fmt::Display for CredentialMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential mapping as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialMapping {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub margin: f64,
    pub issuer: String,
    pub weight: f64,
    pub label: String,
    pub approved: bool,
    pub status: CredentialMappingStatus,
}

impl CredentialMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialMappingStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialMappingStatus) -> bool {
        match self.status {
            CredentialMappingStatus::Draft => {
                return next == CredentialMappingStatus::Active;
            }
            CredentialMappingStatus::Pending => {
                return next == CredentialMappingStatus::Active;
            }
            CredentialMappingStatus::Active => {
                return next == CredentialMappingStatus::Suspended || next == CredentialMappingStatus::Closed;
            }
            CredentialMappingStatus::Suspended => {
                return next == CredentialMappingStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "credential mapping", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.burst as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialMapping {
        CredentialMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            margin: self.margin,
            issuer: self.issuer.clone(),
            weight: self.weight,
            label: self.label.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential mapping in status draft.
pub fn new_credential_mapping(code: &str, name: &str) -> CredentialMapping {
    CredentialMapping {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        margin: 0.0,
        issuer: "".to_string(),
        weight: 0.0,
        label: "".to_string(),
        approved: false,
        status: CredentialMappingStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_credential_mapping("code-291", "name-125");
        assert_eq!(item.status, CredentialMappingStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_credential_mapping("code-291", "name-125");
        if item.move_to(CredentialMappingStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialMappingStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_credential_mapping("code-291", "name-125");
        item.status = CredentialMappingStatus::Closed;
        assert!(
            item.move_to(CredentialMappingStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
