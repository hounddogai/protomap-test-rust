use super::*;

use acme_kit::text;

/// Where a failover cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FailoverCycleStatus {
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

impl std::fmt::Display for FailoverCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCycle {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub floor_value: f64,
    pub attempts: i64,
    pub maximum: f64,
    pub port: i64,
    pub status: FailoverCycleStatus,
}

impl FailoverCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverCycleStatus) -> bool {
        match self.status {
            FailoverCycleStatus::New => {
                return next == FailoverCycleStatus::Queued || next == FailoverCycleStatus::Failed;
            }
            FailoverCycleStatus::Queued => {
                match next {
                    FailoverCycleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverCycleStatus::Running => {
                return next == FailoverCycleStatus::Done || next == FailoverCycleStatus::Failed;
            }
            FailoverCycleStatus::Done => {
                match next {
                    FailoverCycleStatus::Failed => {
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

    pub fn move_to(&mut self, next: FailoverCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "failover cycle", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.maximum - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverCycle {
        FailoverCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            floor_value: self.floor_value,
            attempts: self.attempts,
            maximum: self.maximum,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a failover cycle in status new.
pub fn new_failover_cycle(code: &str, label: &str) -> FailoverCycle {
    FailoverCycle {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        floor_value: 0.0,
        attempts: 0,
        maximum: 0.0,
        port: 0,
        status: FailoverCycleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_failover_cycle("code-187", "label-489");
        assert_eq!(item.status, FailoverCycleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_failover_cycle("code-187", "label-489");
        if item.move_to(FailoverCycleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverCycleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_failover_cycle("code-187", "label-489");
        item.status = FailoverCycleStatus::Failed;
        assert!(item.move_to(FailoverCycleStatus::New).is_err(), "expected move to fail");
    }
}
