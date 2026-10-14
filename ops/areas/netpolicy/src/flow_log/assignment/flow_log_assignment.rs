use super::*;

use acme_kit::text;

/// Where a flow log assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FlowLogAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for FlowLogAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAssignment {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub rate: f64,
    pub name: String,
    pub status: FlowLogAssignmentStatus,
}

impl FlowLogAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogAssignmentStatus) -> bool {
        match self.status {
            FlowLogAssignmentStatus::Draft => {
                return next == FlowLogAssignmentStatus::Pending;
            }
            FlowLogAssignmentStatus::Pending => {
                match next {
                    FlowLogAssignmentStatus::Active | FlowLogAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogAssignmentStatus::Active => {
                match next {
                    FlowLogAssignmentStatus::Suspended | FlowLogAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogAssignmentStatus::Suspended => {
                match next {
                    FlowLogAssignmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: FlowLogAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ceiling + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogAssignment {
        FlowLogAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            rate: self.rate,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a flow log assignment in status draft.
pub fn new_flow_log_assignment(code: &str, label: &str) -> FlowLogAssignment {
    FlowLogAssignment {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        rate: 0.0,
        name: "".to_string(),
        status: FlowLogAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_flow_log_assignment("code-145", "label-267");
        assert_eq!(item.status, FlowLogAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_flow_log_assignment("code-145", "label-267");
        if item.move_to(FlowLogAssignmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogAssignmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_flow_log_assignment("code-145", "label-267");
        item.status = FlowLogAssignmentStatus::Closed;
        assert!(
            item.move_to(FlowLogAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
