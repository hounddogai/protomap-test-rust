use super::*;

use acme_kit::text;

/// Where a sink target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SinkTargetStatus {
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

impl std::fmt::Display for SinkTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkTarget {
    pub code: String,
    pub title: String,
    pub country: String,
    pub retries: i64,
    pub tenant_id: String,
    pub status: SinkTargetStatus,
}

impl SinkTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkTargetStatus) -> bool {
        match self.status {
            SinkTargetStatus::Planned => {
                match next {
                    SinkTargetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkTargetStatus::Scheduled => {
                match next {
                    SinkTargetStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkTargetStatus::InProgress => {
                return next == SinkTargetStatus::Completed || next == SinkTargetStatus::Cancelled;
            }
            SinkTargetStatus::Completed => {
                match next {
                    SinkTargetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SinkTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "sink target", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> SinkTarget {
        SinkTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            retries: self.retries,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a sink target in status planned.
pub fn new_sink_target(code: &str, title: &str) -> SinkTarget {
    SinkTarget {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        retries: 0,
        tenant_id: "".to_string(),
        status: SinkTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_sink_target("code-907", "title-82");
        assert_eq!(item.status, SinkTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_sink_target("code-907", "title-82");
        if item.move_to(SinkTargetStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkTargetStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_sink_target("code-907", "title-82");
        item.status = SinkTargetStatus::Cancelled;
        assert!(
            item.move_to(SinkTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
