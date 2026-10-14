use super::*;

use acme_kit::text;

/// Where a isolation score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IsolationScoreStatus {
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

impl std::fmt::Display for IsolationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScore {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub category: String,
    pub tenant_id: String,
    pub ttl_seconds: i64,
    pub retries: i64,
    pub status: IsolationScoreStatus,
}

impl IsolationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationScoreStatus) -> bool {
        match self.status {
            IsolationScoreStatus::Planned => {
                match next {
                    IsolationScoreStatus::Completed | IsolationScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationScoreStatus::Scheduled => {
                return next == IsolationScoreStatus::InProgress;
            }
            IsolationScoreStatus::InProgress => {
                return next == IsolationScoreStatus::Completed;
            }
            IsolationScoreStatus::Completed => {
                match next {
                    IsolationScoreStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: IsolationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "isolation score", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.revision as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationScore {
        IsolationScore {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            category: self.category.clone(),
            tenant_id: self.tenant_id.clone(),
            ttl_seconds: self.ttl_seconds,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a isolation score in status planned.
pub fn new_isolation_score(code: &str, title: &str) -> IsolationScore {
    IsolationScore {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        category: "".to_string(),
        tenant_id: "".to_string(),
        ttl_seconds: 0,
        retries: 0,
        status: IsolationScoreStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_isolation_score("code-846", "title-743");
        assert_eq!(item.status, IsolationScoreStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_isolation_score("code-846", "title-743");
        if item.move_to(IsolationScoreStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationScoreStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_isolation_score("code-846", "title-743");
        item.status = IsolationScoreStatus::Cancelled;
        assert!(
            item.move_to(IsolationScoreStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
