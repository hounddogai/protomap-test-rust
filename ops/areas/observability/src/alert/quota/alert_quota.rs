use super::*;

use acme_kit::text;

/// Where a alert quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AlertQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AlertQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertQuota {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub unit: String,
    pub country: String,
    pub batch_size: i64,
    pub timezone: String,
    pub updated_at: i64,
    pub fingerprint: String,
    pub version: i64,
    pub status: AlertQuotaStatus,
}

impl AlertQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertQuotaStatus) -> bool {
        match self.status {
            AlertQuotaStatus::New => {
                return next == AlertQuotaStatus::Done || next == AlertQuotaStatus::Failed;
            }
            AlertQuotaStatus::Queued => {
                return next == AlertQuotaStatus::Done;
            }
            AlertQuotaStatus::Running => {
                return next == AlertQuotaStatus::Failed;
            }
            AlertQuotaStatus::Done => {
                return next == AlertQuotaStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.version as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertQuota {
        AlertQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            unit: self.unit.clone(),
            country: self.country.clone(),
            batch_size: self.batch_size,
            timezone: self.timezone.clone(),
            updated_at: self.updated_at,
            fingerprint: self.fingerprint.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a alert quota in status new.
pub fn new_alert_quota(code: &str, label: &str) -> AlertQuota {
    AlertQuota {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        unit: "".to_string(),
        country: "".to_string(),
        batch_size: 0,
        timezone: "".to_string(),
        updated_at: 0,
        fingerprint: "".to_string(),
        version: 0,
        status: AlertQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_alert_quota("code-131", "label-374");
        assert_eq!(item.status, AlertQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_alert_quota("code-131", "label-374");
        if item.move_to(AlertQuotaStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertQuotaStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_alert_quota("code-131", "label-374");
        item.status = AlertQuotaStatus::Failed;
        assert!(item.move_to(AlertQuotaStatus::New).is_err(), "expected move to fail");
    }
}
