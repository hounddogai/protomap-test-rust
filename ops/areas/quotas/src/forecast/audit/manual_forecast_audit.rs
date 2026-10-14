use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual forecast audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualForecastAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualForecastAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ManualForecastAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual forecast audit as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastAudit {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub key_id: String,
    pub timezone: String,
    pub ttl_seconds: i64,
    pub region: String,
    pub window_days: i64,
    pub weight: f64,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub status: ManualForecastAuditStatus,
}

impl ManualForecastAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualForecastAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualForecastAuditStatus) -> bool {
        match self.status {
            ManualForecastAuditStatus::Open => {
                match next {
                    ManualForecastAuditStatus::Resolved | ManualForecastAuditStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualForecastAuditStatus::InReview => {
                return next == ManualForecastAuditStatus::Resolved;
            }
            ManualForecastAuditStatus::Resolved => {
                return next == ManualForecastAuditStatus::Archived;
            }
            ManualForecastAuditStatus::Reopened => {
                match next {
                    ManualForecastAuditStatus::Archived => {
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

    pub fn move_to(&mut self, next: ManualForecastAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual forecast audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual forecast audit", self.code)
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

    pub fn projected_volume(&self) -> f64 {
        ((self.window_days as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualForecastAudit {
        ManualForecastAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            key_id: self.key_id.clone(),
            timezone: self.timezone.clone(),
            ttl_seconds: self.ttl_seconds,
            region: self.region.clone(),
            window_days: self.window_days,
            weight: self.weight,
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual forecast audit in status open.
pub fn new_manual_forecast_audit(code: &str, name: &str) -> ManualForecastAudit {
    ManualForecastAudit {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        key_id: "".to_string(),
        timezone: "".to_string(),
        ttl_seconds: 0,
        region: "".to_string(),
        window_days: 0,
        weight: 0.0,
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        status: ManualForecastAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_forecast_audit("code-867", "name-536");
        assert_eq!(item.status, ManualForecastAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_manual_forecast_audit("code-867", "name-536");
        if item.move_to(ManualForecastAuditStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualForecastAuditStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_forecast_audit("code-867", "name-536");
        item.status = ManualForecastAuditStatus::Archived;
        assert!(
            item.move_to(ManualForecastAuditStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
