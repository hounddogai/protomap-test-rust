use super::*;

use acme_kit::text;

/// Where a flow log score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FlowLogScoreStatus {
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

impl std::fmt::Display for FlowLogScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One flow log score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScore {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub segment: String,
    pub tenant_id: String,
    pub locale: String,
    pub retries: i64,
    pub quantity: i64,
    pub sequence: i64,
    pub priority: i64,
    pub status: FlowLogScoreStatus,
}

impl FlowLogScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogScoreStatus) -> bool {
        match self.status {
            FlowLogScoreStatus::Open => {
                match next {
                    FlowLogScoreStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogScoreStatus::InReview => {
                return next == FlowLogScoreStatus::Resolved || next == FlowLogScoreStatus::Reopened;
            }
            FlowLogScoreStatus::Resolved => {
                match next {
                    FlowLogScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogScoreStatus::Reopened => {
                match next {
                    FlowLogScoreStatus::Archived => {
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

    pub fn move_to(&mut self, next: FlowLogScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.starts_at as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogScore {
        FlowLogScore {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            segment: self.segment.clone(),
            tenant_id: self.tenant_id.clone(),
            locale: self.locale.clone(),
            retries: self.retries,
            quantity: self.quantity,
            sequence: self.sequence,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a flow log score in status open.
pub fn new_flow_log_score(code: &str, title: &str) -> FlowLogScore {
    FlowLogScore {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        segment: "".to_string(),
        tenant_id: "".to_string(),
        locale: "".to_string(),
        retries: 0,
        quantity: 0,
        sequence: 0,
        priority: 0,
        status: FlowLogScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_flow_log_score("code-521", "title-282");
        assert_eq!(item.status, FlowLogScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_flow_log_score("code-521", "title-282");
        if item.move_to(FlowLogScoreStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogScoreStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_flow_log_score("code-521", "title-282");
        item.status = FlowLogScoreStatus::Archived;
        assert!(item.move_to(FlowLogScoreStatus::Open).is_err(), "expected move to fail");
    }
}
