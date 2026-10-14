use super::*;

use acme_kit::text;

/// Where a flow log baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FlowLogBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for FlowLogBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBaseline {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub margin: f64,
    pub host: String,
    pub status: FlowLogBaselineStatus,
}

impl FlowLogBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogBaselineStatus) -> bool {
        match self.status {
            FlowLogBaselineStatus::Open => {
                return next == FlowLogBaselineStatus::InReview;
            }
            FlowLogBaselineStatus::InReview => {
                match next {
                    FlowLogBaselineStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogBaselineStatus::Resolved => {
                match next {
                    FlowLogBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogBaselineStatus::Reopened => {
                return next == FlowLogBaselineStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log baseline", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.batch_size as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogBaseline {
        FlowLogBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            margin: self.margin,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a flow log baseline in status open.
pub fn new_flow_log_baseline(code: &str, name: &str) -> FlowLogBaseline {
    FlowLogBaseline {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        margin: 0.0,
        host: "".to_string(),
        status: FlowLogBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_flow_log_baseline("code-458", "name-927");
        assert_eq!(item.status, FlowLogBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_flow_log_baseline("code-458", "name-927");
        if item.move_to(FlowLogBaselineStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogBaselineStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_flow_log_baseline("code-458", "name-927");
        item.status = FlowLogBaselineStatus::Archived;
        assert!(
            item.move_to(FlowLogBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
