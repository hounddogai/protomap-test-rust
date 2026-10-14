use super::*;

use acme_kit::text;

/// Where a rolling zone quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingZoneQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingZoneQuotaStatus {
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

impl std::fmt::Display for RollingZoneQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling zone quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneQuota {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub threshold: f64,
    pub locale: String,
    pub status: RollingZoneQuotaStatus,
}

impl RollingZoneQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingZoneQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingZoneQuotaStatus) -> bool {
        match self.status {
            RollingZoneQuotaStatus::New => {
                return next == RollingZoneQuotaStatus::Failed;
            }
            RollingZoneQuotaStatus::Queued => {
                return next == RollingZoneQuotaStatus::Running || next == RollingZoneQuotaStatus::Failed;
            }
            RollingZoneQuotaStatus::Running => {
                match next {
                    RollingZoneQuotaStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingZoneQuotaStatus::Done => {
                return next == RollingZoneQuotaStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingZoneQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling zone quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> RollingZoneQuota {
        RollingZoneQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            threshold: self.threshold,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rolling zone quota in status new.
pub fn new_rolling_zone_quota(code: &str, name: &str) -> RollingZoneQuota {
    RollingZoneQuota {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        threshold: 0.0,
        locale: "".to_string(),
        status: RollingZoneQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_zone_quota("code-461", "name-508");
        assert_eq!(item.status, RollingZoneQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_rolling_zone_quota("code-461", "name-508");
        if item.move_to(RollingZoneQuotaStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingZoneQuotaStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_zone_quota("code-461", "name-508");
        item.status = RollingZoneQuotaStatus::Failed;
        assert!(
            item.move_to(RollingZoneQuotaStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
