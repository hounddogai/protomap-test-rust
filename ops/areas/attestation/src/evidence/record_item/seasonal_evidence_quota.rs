use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal evidence quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalEvidenceQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalEvidenceQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalEvidenceQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal evidence quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceQuota {
    pub code: String,
    pub name: String,
    pub category: String,
    pub created_at: i64,
    pub quota_units: i64,
    pub cipher: String,
    pub attempts: i64,
    pub port: i64,
    pub timezone: String,
    pub key_id: String,
    pub attributes: BTreeMap<String, String>,
    pub status: SeasonalEvidenceQuotaStatus,
}

impl SeasonalEvidenceQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalEvidenceQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalEvidenceQuotaStatus) -> bool {
        match self.status {
            SeasonalEvidenceQuotaStatus::Proposed => {
                match next {
                    SeasonalEvidenceQuotaStatus::Approved | SeasonalEvidenceQuotaStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEvidenceQuotaStatus::Approved => {
                return next == SeasonalEvidenceQuotaStatus::Withdrawn;
            }
            SeasonalEvidenceQuotaStatus::Rejected => {
                return next == SeasonalEvidenceQuotaStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalEvidenceQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal evidence quota {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
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

    pub fn weighted_value(&self) -> f64 {
        ((self.port as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalEvidenceQuota {
        SeasonalEvidenceQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            created_at: self.created_at,
            quota_units: self.quota_units,
            cipher: self.cipher.clone(),
            attempts: self.attempts,
            port: self.port,
            timezone: self.timezone.clone(),
            key_id: self.key_id.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a seasonal evidence quota in status proposed.
pub fn new_seasonal_evidence_quota(code: &str, name: &str) -> SeasonalEvidenceQuota {
    SeasonalEvidenceQuota {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        created_at: 0,
        quota_units: 0,
        cipher: "".to_string(),
        attempts: 0,
        port: 0,
        timezone: "".to_string(),
        key_id: "".to_string(),
        attributes: BTreeMap::new(),
        status: SeasonalEvidenceQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_evidence_quota("code-905", "name-388");
        assert_eq!(item.status, SeasonalEvidenceQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_seasonal_evidence_quota("code-905", "name-388");
        if item.move_to(SeasonalEvidenceQuotaStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalEvidenceQuotaStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_evidence_quota("code-905", "name-388");
        item.status = SeasonalEvidenceQuotaStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalEvidenceQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
