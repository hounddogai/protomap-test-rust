use super::*;

use acme_kit::text;

/// Where a scheduled suspension quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSuspensionQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledSuspensionQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledSuspensionQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled suspension quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionQuota {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub currency: String,
    pub expires_at: i64,
    pub status: ScheduledSuspensionQuotaStatus,
}

impl ScheduledSuspensionQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSuspensionQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSuspensionQuotaStatus) -> bool {
        match self.status {
            ScheduledSuspensionQuotaStatus::Created => {
                return next == ScheduledSuspensionQuotaStatus::Validated
                    || next == ScheduledSuspensionQuotaStatus::Retired;
            }
            ScheduledSuspensionQuotaStatus::Validated => {
                return next == ScheduledSuspensionQuotaStatus::Published
                    || next == ScheduledSuspensionQuotaStatus::Retired;
            }
            ScheduledSuspensionQuotaStatus::Published => {
                match next {
                    ScheduledSuspensionQuotaStatus::Retired => {
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

    pub fn move_to(&mut self, next: ScheduledSuspensionQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled suspension quota {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled suspension quota", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ends_at as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSuspensionQuota {
        ScheduledSuspensionQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            currency: self.currency.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled suspension quota in status created.
pub fn new_scheduled_suspension_quota(code: &str, name: &str) -> ScheduledSuspensionQuota {
    ScheduledSuspensionQuota {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        currency: "".to_string(),
        expires_at: 0,
        status: ScheduledSuspensionQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_suspension_quota("code-738", "name-655");
        assert_eq!(item.status, ScheduledSuspensionQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_scheduled_suspension_quota("code-738", "name-655");
        if item.move_to(ScheduledSuspensionQuotaStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSuspensionQuotaStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_suspension_quota("code-738", "name-655");
        item.status = ScheduledSuspensionQuotaStatus::Retired;
        assert!(
            item.move_to(ScheduledSuspensionQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
