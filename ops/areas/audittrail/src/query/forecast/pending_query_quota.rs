use super::*;

use acme_kit::text;

/// Where a pending query quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingQueryQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingQueryQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingQueryQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending query quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryQuota {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub rate: f64,
    pub retries: i64,
    pub principal: String,
    pub quota_units: i64,
    pub rotation_days: i64,
    pub status: PendingQueryQuotaStatus,
}

impl PendingQueryQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingQueryQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingQueryQuotaStatus) -> bool {
        match self.status {
            PendingQueryQuotaStatus::Created => {
                return next == PendingQueryQuotaStatus::Validated;
            }
            PendingQueryQuotaStatus::Validated => {
                match next {
                    PendingQueryQuotaStatus::Published | PendingQueryQuotaStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingQueryQuotaStatus::Published => {
                match next {
                    PendingQueryQuotaStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingQueryQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending query quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending query quota", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.quota_units as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingQueryQuota {
        PendingQueryQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            rate: self.rate,
            retries: self.retries,
            principal: self.principal.clone(),
            quota_units: self.quota_units,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending query quota in status created.
pub fn new_pending_query_quota(code: &str, title: &str) -> PendingQueryQuota {
    PendingQueryQuota {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        rate: 0.0,
        retries: 0,
        principal: "".to_string(),
        quota_units: 0,
        rotation_days: 0,
        status: PendingQueryQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_query_quota("code-923", "title-94");
        assert_eq!(item.status, PendingQueryQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_pending_query_quota("code-923", "title-94");
        if item.move_to(PendingQueryQuotaStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingQueryQuotaStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_query_quota("code-923", "title-94");
        item.status = PendingQueryQuotaStatus::Retired;
        assert!(
            item.move_to(PendingQueryQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
