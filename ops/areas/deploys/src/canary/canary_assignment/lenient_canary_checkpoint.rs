use super::*;

use acme_kit::text;

/// Where a lenient canary checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientCanaryCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientCanaryCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LenientCanaryCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient canary checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryCheckpoint {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub ends_at: i64,
    pub attempts: i64,
    pub discount: f64,
    pub limit: i64,
    pub maximum: f64,
    pub status: LenientCanaryCheckpointStatus,
}

impl LenientCanaryCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientCanaryCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientCanaryCheckpointStatus) -> bool {
        match self.status {
            LenientCanaryCheckpointStatus::Planned => {
                return next == LenientCanaryCheckpointStatus::Completed
                    || next == LenientCanaryCheckpointStatus::Cancelled;
            }
            LenientCanaryCheckpointStatus::Scheduled => {
                return next == LenientCanaryCheckpointStatus::InProgress;
            }
            LenientCanaryCheckpointStatus::InProgress => {
                return next == LenientCanaryCheckpointStatus::Completed
                    || next == LenientCanaryCheckpointStatus::Cancelled;
            }
            LenientCanaryCheckpointStatus::Completed => {
                match next {
                    LenientCanaryCheckpointStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientCanaryCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient canary checkpoint {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient canary checkpoint", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ends_at as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientCanaryCheckpoint {
        LenientCanaryCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            ends_at: self.ends_at,
            attempts: self.attempts,
            discount: self.discount,
            limit: self.limit,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient canary checkpoint in status planned.
pub fn new_lenient_canary_checkpoint(code: &str, label: &str) -> LenientCanaryCheckpoint {
    LenientCanaryCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        ends_at: 0,
        attempts: 0,
        discount: 0.0,
        limit: 0,
        maximum: 0.0,
        status: LenientCanaryCheckpointStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_canary_checkpoint("code-991", "label-616");
        assert_eq!(item.status, LenientCanaryCheckpointStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_lenient_canary_checkpoint("code-991", "label-616");
        if item.move_to(LenientCanaryCheckpointStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientCanaryCheckpointStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_canary_checkpoint("code-991", "label-616");
        item.status = LenientCanaryCheckpointStatus::Cancelled;
        assert!(
            item.move_to(LenientCanaryCheckpointStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
