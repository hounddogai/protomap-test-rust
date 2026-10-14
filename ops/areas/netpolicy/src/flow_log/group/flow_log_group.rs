use super::*;

use acme_kit::text;

/// Where a flow log group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FlowLogGroupStatus {
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

impl std::fmt::Display for FlowLogGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a flow log group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogGroup {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub floor_value: f64,
    pub amount: f64,
    pub cost: f64,
    pub status: FlowLogGroupStatus,
}

impl FlowLogGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogGroupStatus) -> bool {
        match self.status {
            FlowLogGroupStatus::Planned => {
                return next == FlowLogGroupStatus::InProgress;
            }
            FlowLogGroupStatus::Scheduled => {
                return next == FlowLogGroupStatus::InProgress || next == FlowLogGroupStatus::Completed;
            }
            FlowLogGroupStatus::InProgress => {
                return next == FlowLogGroupStatus::Cancelled;
            }
            FlowLogGroupStatus::Completed => {
                return next == FlowLogGroupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log group", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.floor_value + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogGroup {
        FlowLogGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            floor_value: self.floor_value,
            amount: self.amount,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a flow log group in status planned.
pub fn new_flow_log_group(code: &str, title: &str) -> FlowLogGroup {
    FlowLogGroup {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        floor_value: 0.0,
        amount: 0.0,
        cost: 0.0,
        status: FlowLogGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_flow_log_group("code-561", "title-872");
        assert_eq!(item.status, FlowLogGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_flow_log_group("code-561", "title-872");
        if item.move_to(FlowLogGroupStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogGroupStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_flow_log_group("code-561", "title-872");
        item.status = FlowLogGroupStatus::Cancelled;
        assert!(
            item.move_to(FlowLogGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
