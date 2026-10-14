use super::*;

use acme_kit::text;

/// Where a flow log segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FlowLogSegmentStatus {
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

impl std::fmt::Display for FlowLogSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSegment {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub priority: i64,
    pub approved: bool,
    pub active: bool,
    pub status: FlowLogSegmentStatus,
}

impl FlowLogSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogSegmentStatus) -> bool {
        match self.status {
            FlowLogSegmentStatus::Planned => {
                match next {
                    FlowLogSegmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogSegmentStatus::Scheduled => {
                match next {
                    FlowLogSegmentStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogSegmentStatus::InProgress => {
                return next == FlowLogSegmentStatus::Completed || next == FlowLogSegmentStatus::Cancelled;
            }
            FlowLogSegmentStatus::Completed => {
                match next {
                    FlowLogSegmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FlowLogSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.priority as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogSegment {
        FlowLogSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            priority: self.priority,
            approved: self.approved,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a flow log segment in status planned.
pub fn new_flow_log_segment(code: &str, title: &str) -> FlowLogSegment {
    FlowLogSegment {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        priority: 0,
        approved: false,
        active: false,
        status: FlowLogSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_flow_log_segment("code-951", "title-928");
        assert_eq!(item.status, FlowLogSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_flow_log_segment("code-951", "title-928");
        if item.move_to(FlowLogSegmentStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogSegmentStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_flow_log_segment("code-951", "title-928");
        item.status = FlowLogSegmentStatus::Cancelled;
        assert!(
            item.move_to(FlowLogSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
