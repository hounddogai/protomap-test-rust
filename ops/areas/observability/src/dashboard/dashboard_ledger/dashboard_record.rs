use super::*;

use acme_kit::text;

/// Where a dashboard record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DashboardRecordStatus {
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

impl std::fmt::Display for DashboardRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRecord {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub maximum: f64,
    pub quantity: i64,
    pub grace_days: i64,
    pub expires_at: i64,
    pub scope: String,
    pub ends_at: i64,
    pub budget: f64,
    pub status: DashboardRecordStatus,
}

impl DashboardRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardRecordStatus) -> bool {
        match self.status {
            DashboardRecordStatus::New => {
                match next {
                    DashboardRecordStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardRecordStatus::Queued => {
                return next == DashboardRecordStatus::Running || next == DashboardRecordStatus::Done;
            }
            DashboardRecordStatus::Running => {
                match next {
                    DashboardRecordStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardRecordStatus::Done => {
                match next {
                    DashboardRecordStatus::Failed => {
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

    pub fn move_to(&mut self, next: DashboardRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.grace_days as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardRecord {
        DashboardRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            maximum: self.maximum,
            quantity: self.quantity,
            grace_days: self.grace_days,
            expires_at: self.expires_at,
            scope: self.scope.clone(),
            ends_at: self.ends_at,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a dashboard record in status new.
pub fn new_dashboard_record(code: &str, name: &str) -> DashboardRecord {
    DashboardRecord {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        maximum: 0.0,
        quantity: 0,
        grace_days: 0,
        expires_at: 0,
        scope: "".to_string(),
        ends_at: 0,
        budget: 0.0,
        status: DashboardRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_dashboard_record("code-236", "name-184");
        assert_eq!(item.status, DashboardRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_dashboard_record("code-236", "name-184");
        if item.move_to(DashboardRecordStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardRecordStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_dashboard_record("code-236", "name-184");
        item.status = DashboardRecordStatus::Failed;
        assert!(
            item.move_to(DashboardRecordStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
