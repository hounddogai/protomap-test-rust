use super::*;

use acme_kit::text;

/// Where a rolling isolation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingIsolationTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingIsolationTargetStatus {
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

impl std::fmt::Display for RollingIsolationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling isolation target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationTarget {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub summary: String,
    pub window_days: i64,
    pub budget: f64,
    pub updated_at: i64,
    pub issuer: String,
    pub tenant_id: String,
    pub priority: i64,
    pub status: RollingIsolationTargetStatus,
}

impl RollingIsolationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingIsolationTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingIsolationTargetStatus) -> bool {
        match self.status {
            RollingIsolationTargetStatus::New => {
                match next {
                    RollingIsolationTargetStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingIsolationTargetStatus::Queued => {
                match next {
                    RollingIsolationTargetStatus::Done | RollingIsolationTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingIsolationTargetStatus::Running => {
                return next == RollingIsolationTargetStatus::Failed;
            }
            RollingIsolationTargetStatus::Done => {
                match next {
                    RollingIsolationTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingIsolationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling isolation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling isolation target", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.priority as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingIsolationTarget {
        RollingIsolationTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            window_days: self.window_days,
            budget: self.budget,
            updated_at: self.updated_at,
            issuer: self.issuer.clone(),
            tenant_id: self.tenant_id.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling isolation target in status new.
pub fn new_rolling_isolation_target(code: &str, name: &str) -> RollingIsolationTarget {
    RollingIsolationTarget {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        summary: "".to_string(),
        window_days: 0,
        budget: 0.0,
        updated_at: 0,
        issuer: "".to_string(),
        tenant_id: "".to_string(),
        priority: 0,
        status: RollingIsolationTargetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_isolation_target("code-633", "name-624");
        assert_eq!(item.status, RollingIsolationTargetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rolling_isolation_target("code-633", "name-624");
        if item.move_to(RollingIsolationTargetStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingIsolationTargetStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_isolation_target("code-633", "name-624");
        item.status = RollingIsolationTargetStatus::Failed;
        assert!(
            item.move_to(RollingIsolationTargetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
