use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default identity quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultIdentityQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultIdentityQuotaStatus {
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

impl std::fmt::Display for DefaultIdentityQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default identity quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityQuota {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub cipher: String,
    pub expires_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub amount: f64,
    pub floor_value: f64,
    pub label: String,
    pub namespace: String,
    pub status: DefaultIdentityQuotaStatus,
}

impl DefaultIdentityQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultIdentityQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultIdentityQuotaStatus) -> bool {
        match self.status {
            DefaultIdentityQuotaStatus::Draft => {
                match next {
                    DefaultIdentityQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultIdentityQuotaStatus::Pending => {
                match next {
                    DefaultIdentityQuotaStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultIdentityQuotaStatus::Active => {
                return next == DefaultIdentityQuotaStatus::Suspended;
            }
            DefaultIdentityQuotaStatus::Suspended => {
                return next == DefaultIdentityQuotaStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultIdentityQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default identity quota {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_value(&self) -> f64 {
        ((self.expires_at as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultIdentityQuota {
        DefaultIdentityQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            cipher: self.cipher.clone(),
            expires_at: self.expires_at,
            attributes: self.attributes.clone(),
            amount: self.amount,
            floor_value: self.floor_value,
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a default identity quota in status draft.
pub fn new_default_identity_quota(code: &str, name: &str) -> DefaultIdentityQuota {
    DefaultIdentityQuota {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        cipher: "".to_string(),
        expires_at: 0,
        attributes: BTreeMap::new(),
        amount: 0.0,
        floor_value: 0.0,
        label: "".to_string(),
        namespace: "".to_string(),
        status: DefaultIdentityQuotaStatus::Draft,
    }
}
