use super::*;

use acme_kit::text;

/// Where a runbook quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RunbookQuotaStatus {
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

impl std::fmt::Display for RunbookQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookQuota {
    pub code: String,
    pub label: String,
    pub region: String,
    pub duration_minutes: i64,
    pub locale: String,
    pub version: i64,
    pub country: String,
    pub quota_units: i64,
    pub status: RunbookQuotaStatus,
}

impl RunbookQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookQuotaStatus) -> bool {
        match self.status {
            RunbookQuotaStatus::Draft => {
                match next {
                    RunbookQuotaStatus::Pending | RunbookQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookQuotaStatus::Pending => {
                return next == RunbookQuotaStatus::Active || next == RunbookQuotaStatus::Suspended;
            }
            RunbookQuotaStatus::Active => {
                return next == RunbookQuotaStatus::Closed;
            }
            RunbookQuotaStatus::Suspended => {
                return next == RunbookQuotaStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "runbook quota", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.quota_units as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookQuota {
        RunbookQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            duration_minutes: self.duration_minutes,
            locale: self.locale.clone(),
            version: self.version,
            country: self.country.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a runbook quota in status draft.
pub fn new_runbook_quota(code: &str, label: &str) -> RunbookQuota {
    RunbookQuota {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        duration_minutes: 0,
        locale: "".to_string(),
        version: 0,
        country: "".to_string(),
        quota_units: 0,
        status: RunbookQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_runbook_quota("code-554", "label-848");
        assert_eq!(item.status, RunbookQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_runbook_quota("code-554", "label-848");
        if item.move_to(RunbookQuotaStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookQuotaStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_runbook_quota("code-554", "label-848");
        item.status = RunbookQuotaStatus::Closed;
        assert!(
            item.move_to(RunbookQuotaStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
