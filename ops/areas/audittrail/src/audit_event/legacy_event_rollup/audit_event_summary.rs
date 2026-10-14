use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a audit event summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventSummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AuditEventSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AuditEventSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSummary {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub burst: i64,
    pub starts_at: i64,
    pub rate: f64,
    pub discount: f64,
    pub retries: i64,
    pub phone: String,
    pub attributes: BTreeMap<String, String>,
    pub key_id: String,
    pub status: AuditEventSummaryStatus,
}

impl AuditEventSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventSummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventSummaryStatus) -> bool {
        match self.status {
            AuditEventSummaryStatus::Created => {
                return next == AuditEventSummaryStatus::Retired;
            }
            AuditEventSummaryStatus::Validated => {
                match next {
                    AuditEventSummaryStatus::Published | AuditEventSummaryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventSummaryStatus::Published => {
                return next == AuditEventSummaryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventSummary {
        AuditEventSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            burst: self.burst,
            starts_at: self.starts_at,
            rate: self.rate,
            discount: self.discount,
            retries: self.retries,
            phone: self.phone.clone(),
            attributes: self.attributes.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a audit event summary in status created.
pub fn new_audit_event_summary(code: &str, name: &str) -> AuditEventSummary {
    AuditEventSummary {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        burst: 0,
        starts_at: 0,
        rate: 0.0,
        discount: 0.0,
        retries: 0,
        phone: "".to_string(),
        attributes: BTreeMap::new(),
        key_id: "".to_string(),
        status: AuditEventSummaryStatus::Created,
    }
}
