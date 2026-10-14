use super::*;

use acme_kit::text;

/// Where a legacy health check quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyHealthCheckQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyHealthCheckQuotaStatus {
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

impl std::fmt::Display for LegacyHealthCheckQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy health check quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyHealthCheckQuota {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub name: String,
    pub rotation_days: i64,
    pub margin: f64,
    pub host: String,
    pub window_days: i64,
    pub summary: String,
    pub status: LegacyHealthCheckQuotaStatus,
}

impl LegacyHealthCheckQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyHealthCheckQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyHealthCheckQuotaStatus) -> bool {
        match self.status {
            LegacyHealthCheckQuotaStatus::New => {
                return next == LegacyHealthCheckQuotaStatus::Queued || next == LegacyHealthCheckQuotaStatus::Failed;
            }
            LegacyHealthCheckQuotaStatus::Queued => {
                match next {
                    LegacyHealthCheckQuotaStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyHealthCheckQuotaStatus::Running => {
                match next {
                    LegacyHealthCheckQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyHealthCheckQuotaStatus::Done => {
                return next == LegacyHealthCheckQuotaStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyHealthCheckQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy health check quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.margin - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyHealthCheckQuota {
        LegacyHealthCheckQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            margin: self.margin,
            host: self.host.clone(),
            window_days: self.window_days,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy health check quota in status new.
pub fn new_legacy_health_check_quota(code: &str, label: &str) -> LegacyHealthCheckQuota {
    LegacyHealthCheckQuota {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        name: "".to_string(),
        rotation_days: 0,
        margin: 0.0,
        host: "".to_string(),
        window_days: 0,
        summary: "".to_string(),
        status: LegacyHealthCheckQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_health_check_quota("code-621", "label-378");
        assert_eq!(item.status, LegacyHealthCheckQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_legacy_health_check_quota("code-621", "label-378");
        if item.move_to(LegacyHealthCheckQuotaStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyHealthCheckQuotaStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_health_check_quota("code-621", "label-378");
        item.status = LegacyHealthCheckQuotaStatus::Failed;
        assert!(
            item.move_to(LegacyHealthCheckQuotaStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
