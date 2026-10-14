use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a capped overage quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedOverageQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedOverageQuotaStatus {
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

impl std::fmt::Display for CappedOverageQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped overage quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageQuota {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub attributes: BTreeMap<String, String>,
    pub attempts: i64,
    pub timezone: String,
    pub status: CappedOverageQuotaStatus,
}

impl CappedOverageQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedOverageQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedOverageQuotaStatus) -> bool {
        match self.status {
            CappedOverageQuotaStatus::Draft => {
                return next == CappedOverageQuotaStatus::Pending;
            }
            CappedOverageQuotaStatus::Pending => {
                return next == CappedOverageQuotaStatus::Active || next == CappedOverageQuotaStatus::Suspended;
            }
            CappedOverageQuotaStatus::Active => {
                return next == CappedOverageQuotaStatus::Suspended || next == CappedOverageQuotaStatus::Closed;
            }
            CappedOverageQuotaStatus::Suspended => {
                return next == CappedOverageQuotaStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedOverageQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped overage quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> CappedOverageQuota {
        CappedOverageQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            attributes: self.attributes.clone(),
            attempts: self.attempts,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped overage quota in status draft.
pub fn new_capped_overage_quota(code: &str, title: &str) -> CappedOverageQuota {
    CappedOverageQuota {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        attributes: BTreeMap::new(),
        attempts: 0,
        timezone: "".to_string(),
        status: CappedOverageQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_overage_quota("code-791", "title-960");
        assert_eq!(item.status, CappedOverageQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_capped_overage_quota("code-791", "title-960");
        if item.move_to(CappedOverageQuotaStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedOverageQuotaStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_overage_quota("code-791", "title-960");
        item.status = CappedOverageQuotaStatus::Closed;
        assert!(
            item.move_to(CappedOverageQuotaStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
