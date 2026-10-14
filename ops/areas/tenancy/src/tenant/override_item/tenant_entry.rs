use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a tenant entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TenantEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TenantEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant entry as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantEntry {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub name: String,
    pub phone: String,
    pub duration_minutes: i64,
    pub region: String,
    pub approved: bool,
    pub principal: String,
    pub amount: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: TenantEntryStatus,
}

impl TenantEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantEntryStatus) -> bool {
        match self.status {
            TenantEntryStatus::Created => {
                return next == TenantEntryStatus::Published;
            }
            TenantEntryStatus::Validated => {
                return next == TenantEntryStatus::Published || next == TenantEntryStatus::Retired;
            }
            TenantEntryStatus::Published => {
                match next {
                    TenantEntryStatus::Retired => {
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

    pub fn move_to(&mut self, next: TenantEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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
        ((self.quota_units as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantEntry {
        TenantEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            name: self.name.clone(),
            phone: self.phone.clone(),
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            approved: self.approved,
            principal: self.principal.clone(),
            amount: self.amount,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tenant entry in status created.
pub fn new_tenant_entry(code: &str, title: &str) -> TenantEntry {
    TenantEntry {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        name: "".to_string(),
        phone: "".to_string(),
        duration_minutes: 0,
        region: "".to_string(),
        approved: false,
        principal: "".to_string(),
        amount: 0.0,
        attributes: BTreeMap::new(),
        status: TenantEntryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tenant_entry("code-244", "title-352");
        assert_eq!(item.status, TenantEntryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_tenant_entry("code-244", "title-352");
        if item.move_to(TenantEntryStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantEntryStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tenant_entry("code-244", "title-352");
        item.status = TenantEntryStatus::Retired;
        assert!(
            item.move_to(TenantEntryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
