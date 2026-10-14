use super::*;

use acme_kit::text;

/// Where a scheduled health check audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledHealthCheckAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledHealthCheckAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ScheduledHealthCheckAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled health check audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckAudit {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub revision: i64,
    pub quantity: i64,
    pub capacity: i64,
    pub minimum: f64,
    pub status: ScheduledHealthCheckAuditStatus,
}

impl ScheduledHealthCheckAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledHealthCheckAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledHealthCheckAuditStatus) -> bool {
        match self.status {
            ScheduledHealthCheckAuditStatus::Idle => {
                return next == ScheduledHealthCheckAuditStatus::Ready
                    || next == ScheduledHealthCheckAuditStatus::Stopped;
            }
            ScheduledHealthCheckAuditStatus::Warming => {
                match next {
                    ScheduledHealthCheckAuditStatus::Ready | ScheduledHealthCheckAuditStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledHealthCheckAuditStatus::Ready => {
                return next == ScheduledHealthCheckAuditStatus::Draining
                    || next == ScheduledHealthCheckAuditStatus::Stopped;
            }
            ScheduledHealthCheckAuditStatus::Draining => {
                match next {
                    ScheduledHealthCheckAuditStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledHealthCheckAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled health check audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled health check audit", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.minimum + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledHealthCheckAudit {
        ScheduledHealthCheckAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            revision: self.revision,
            quantity: self.quantity,
            capacity: self.capacity,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled health check audit in status idle.
pub fn new_scheduled_health_check_audit(code: &str, name: &str) -> ScheduledHealthCheckAudit {
    ScheduledHealthCheckAudit {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        revision: 0,
        quantity: 0,
        capacity: 0,
        minimum: 0.0,
        status: ScheduledHealthCheckAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_health_check_audit("code-941", "name-872");
        assert_eq!(item.status, ScheduledHealthCheckAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_scheduled_health_check_audit("code-941", "name-872");
        if item.move_to(ScheduledHealthCheckAuditStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledHealthCheckAuditStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_health_check_audit("code-941", "name-872");
        item.status = ScheduledHealthCheckAuditStatus::Stopped;
        assert!(
            item.move_to(ScheduledHealthCheckAuditStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
