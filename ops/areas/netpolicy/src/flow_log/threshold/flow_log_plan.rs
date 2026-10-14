use super::*;

use acme_kit::text;

/// Where a flow log plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogPlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FlowLogPlanStatus {
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

impl std::fmt::Display for FlowLogPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPlan {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub contact_email: String,
    pub quantity: i64,
    pub visible: bool,
    pub priority: i64,
    pub summary: String,
    pub created_at: i64,
    pub status: FlowLogPlanStatus,
}

impl FlowLogPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogPlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogPlanStatus) -> bool {
        match self.status {
            FlowLogPlanStatus::Draft => {
                match next {
                    FlowLogPlanStatus::Pending | FlowLogPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogPlanStatus::Pending => {
                return next == FlowLogPlanStatus::Active || next == FlowLogPlanStatus::Closed;
            }
            FlowLogPlanStatus::Active => {
                return next == FlowLogPlanStatus::Closed;
            }
            FlowLogPlanStatus::Suspended => {
                match next {
                    FlowLogPlanStatus::Closed => {
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

    pub fn move_to(&mut self, next: FlowLogPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log plan {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "flow log plan", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogPlan {
        FlowLogPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            contact_email: self.contact_email.clone(),
            quantity: self.quantity,
            visible: self.visible,
            priority: self.priority,
            summary: self.summary.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a flow log plan in status draft.
pub fn new_flow_log_plan(code: &str, label: &str) -> FlowLogPlan {
    FlowLogPlan {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        contact_email: "".to_string(),
        quantity: 0,
        visible: false,
        priority: 0,
        summary: "".to_string(),
        created_at: 0,
        status: FlowLogPlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_flow_log_plan("code-467", "label-86");
        assert_eq!(item.status, FlowLogPlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_flow_log_plan("code-467", "label-86");
        if item.move_to(FlowLogPlanStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogPlanStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_flow_log_plan("code-467", "label-86");
        item.status = FlowLogPlanStatus::Closed;
        assert!(item.move_to(FlowLogPlanStatus::Draft).is_err(), "expected move to fail");
    }
}
