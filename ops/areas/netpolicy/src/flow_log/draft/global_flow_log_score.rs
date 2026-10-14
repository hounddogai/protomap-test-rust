use super::*;

use acme_kit::text;

/// Where a global flow log score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalFlowLogScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalFlowLogScoreStatus {
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

impl std::fmt::Display for GlobalFlowLogScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global flow log score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogScore {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub contact_email: String,
    pub window_days: i64,
    pub owner_id: String,
    pub quantity: i64,
    pub duration_minutes: i64,
    pub status: GlobalFlowLogScoreStatus,
}

impl GlobalFlowLogScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalFlowLogScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalFlowLogScoreStatus) -> bool {
        match self.status {
            GlobalFlowLogScoreStatus::Open => {
                match next {
                    GlobalFlowLogScoreStatus::Reopened | GlobalFlowLogScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFlowLogScoreStatus::InReview => {
                match next {
                    GlobalFlowLogScoreStatus::Reopened | GlobalFlowLogScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFlowLogScoreStatus::Resolved => {
                match next {
                    GlobalFlowLogScoreStatus::Reopened | GlobalFlowLogScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFlowLogScoreStatus::Reopened => {
                return next == GlobalFlowLogScoreStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalFlowLogScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global flow log score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalFlowLogScore {
        GlobalFlowLogScore {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            contact_email: self.contact_email.clone(),
            window_days: self.window_days,
            owner_id: self.owner_id.clone(),
            quantity: self.quantity,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global flow log score in status open.
pub fn new_global_flow_log_score(code: &str, name: &str) -> GlobalFlowLogScore {
    GlobalFlowLogScore {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        contact_email: "".to_string(),
        window_days: 0,
        owner_id: "".to_string(),
        quantity: 0,
        duration_minutes: 0,
        status: GlobalFlowLogScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_global_flow_log_score("code-546", "name-634");
        assert_eq!(item.status, GlobalFlowLogScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_global_flow_log_score("code-546", "name-634");
        if item.move_to(GlobalFlowLogScoreStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalFlowLogScoreStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_global_flow_log_score("code-546", "name-634");
        item.status = GlobalFlowLogScoreStatus::Archived;
        assert!(
            item.move_to(GlobalFlowLogScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
