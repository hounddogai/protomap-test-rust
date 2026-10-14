use super::*;

use acme_kit::text;

/// Where a tenant override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TenantOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TenantOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant override as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantOverride {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub quantity: i64,
    pub tags: Vec<String>,
    pub channel: String,
    pub contact_email: String,
    pub status: TenantOverrideStatus,
}

impl TenantOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantOverrideStatus) -> bool {
        match self.status {
            TenantOverrideStatus::Requested => {
                match next {
                    TenantOverrideStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantOverrideStatus::Granted => {
                match next {
                    TenantOverrideStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantOverrideStatus::Revoked => {
                return next == TenantOverrideStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> TenantOverride {
        TenantOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            tags: self.tags.clone(),
            channel: self.channel.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a tenant override in status requested.
pub fn new_tenant_override(code: &str, name: &str) -> TenantOverride {
    TenantOverride {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        quantity: 0,
        tags: Vec::new(),
        channel: "".to_string(),
        contact_email: "".to_string(),
        status: TenantOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tenant_override("code-30", "name-916");
        assert_eq!(item.status, TenantOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_tenant_override("code-30", "name-916");
        if item.move_to(TenantOverrideStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantOverrideStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tenant_override("code-30", "name-916");
        item.status = TenantOverrideStatus::Expired;
        assert!(
            item.move_to(TenantOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
