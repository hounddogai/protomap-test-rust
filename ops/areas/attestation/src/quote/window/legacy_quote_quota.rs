use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a legacy quote quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyQuoteQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyQuoteQuotaStatus {
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

impl std::fmt::Display for LegacyQuoteQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy quote quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteQuota {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub limit: i64,
    pub title: String,
    pub cluster: String,
    pub host: String,
    pub pinned: bool,
    pub ends_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub starts_at: i64,
    pub status: LegacyQuoteQuotaStatus,
}

impl LegacyQuoteQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyQuoteQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyQuoteQuotaStatus) -> bool {
        match self.status {
            LegacyQuoteQuotaStatus::Draft => {
                return next == LegacyQuoteQuotaStatus::Closed;
            }
            LegacyQuoteQuotaStatus::Pending => {
                return next == LegacyQuoteQuotaStatus::Closed;
            }
            LegacyQuoteQuotaStatus::Active => {
                return next == LegacyQuoteQuotaStatus::Closed;
            }
            LegacyQuoteQuotaStatus::Suspended => {
                return next == LegacyQuoteQuotaStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyQuoteQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy quote quota {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy quote quota", self.code)
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

    pub fn projected_exposure(&self) -> f64 {
        ((self.starts_at as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyQuoteQuota {
        LegacyQuoteQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            limit: self.limit,
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            host: self.host.clone(),
            pinned: self.pinned,
            ends_at: self.ends_at,
            attributes: self.attributes.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy quote quota in status draft.
pub fn new_legacy_quote_quota(code: &str, label: &str) -> LegacyQuoteQuota {
    LegacyQuoteQuota {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        limit: 0,
        title: "".to_string(),
        cluster: "".to_string(),
        host: "".to_string(),
        pinned: false,
        ends_at: 0,
        attributes: BTreeMap::new(),
        starts_at: 0,
        status: LegacyQuoteQuotaStatus::Draft,
    }
}
