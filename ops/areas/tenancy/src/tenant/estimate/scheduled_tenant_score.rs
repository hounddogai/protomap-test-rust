use super::*;

use acme_kit::text;

/// Where a scheduled tenant score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTenantScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledTenantScoreStatus {
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

impl std::fmt::Display for ScheduledTenantScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled tenant score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantScore {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub capacity: i64,
    pub timezone: String,
    pub status: ScheduledTenantScoreStatus,
}

impl ScheduledTenantScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTenantScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTenantScoreStatus) -> bool {
        match self.status {
            ScheduledTenantScoreStatus::New => {
                match next {
                    ScheduledTenantScoreStatus::Queued | ScheduledTenantScoreStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTenantScoreStatus::Queued => {
                match next {
                    ScheduledTenantScoreStatus::Running | ScheduledTenantScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTenantScoreStatus::Running => {
                match next {
                    ScheduledTenantScoreStatus::Done | ScheduledTenantScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTenantScoreStatus::Done => {
                return next == ScheduledTenantScoreStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledTenantScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled tenant score {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.capacity as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTenantScore {
        ScheduledTenantScore {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            capacity: self.capacity,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled tenant score in status new.
pub fn new_scheduled_tenant_score(code: &str, name: &str) -> ScheduledTenantScore {
    ScheduledTenantScore {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        capacity: 0,
        timezone: "".to_string(),
        status: ScheduledTenantScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_tenant_score("code-565", "name-899");
        assert_eq!(item.status, ScheduledTenantScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_scheduled_tenant_score("code-565", "name-899");
        if item.move_to(ScheduledTenantScoreStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTenantScoreStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_tenant_score("code-565", "name-899");
        item.status = ScheduledTenantScoreStatus::Failed;
        assert!(
            item.move_to(ScheduledTenantScoreStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
