use super::*;

use acme_kit::text;

/// Where a weighted flow log mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFlowLogMappingStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedFlowLogMappingStatus {
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

impl std::fmt::Display for WeightedFlowLogMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted flow log mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogMapping {
    pub code: String,
    pub name: String,
    pub title: String,
    pub cipher: String,
    pub approved: bool,
    pub audience: String,
    pub priority: i64,
    pub status: WeightedFlowLogMappingStatus,
}

impl WeightedFlowLogMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFlowLogMappingStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFlowLogMappingStatus) -> bool {
        match self.status {
            WeightedFlowLogMappingStatus::Open => {
                return next == WeightedFlowLogMappingStatus::InReview
                    || next == WeightedFlowLogMappingStatus::Archived;
            }
            WeightedFlowLogMappingStatus::InReview => {
                match next {
                    WeightedFlowLogMappingStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFlowLogMappingStatus::Resolved => {
                match next {
                    WeightedFlowLogMappingStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFlowLogMappingStatus::Reopened => {
                return next == WeightedFlowLogMappingStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedFlowLogMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted flow log mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> WeightedFlowLogMapping {
        WeightedFlowLogMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            approved: self.approved,
            audience: self.audience.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted flow log mapping in status open.
pub fn new_weighted_flow_log_mapping(code: &str, name: &str) -> WeightedFlowLogMapping {
    WeightedFlowLogMapping {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        cipher: "".to_string(),
        approved: false,
        audience: "".to_string(),
        priority: 0,
        status: WeightedFlowLogMappingStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_flow_log_mapping("code-609", "name-730");
        assert_eq!(item.status, WeightedFlowLogMappingStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_weighted_flow_log_mapping("code-609", "name-730");
        if item.move_to(WeightedFlowLogMappingStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedFlowLogMappingStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_flow_log_mapping("code-609", "name-730");
        item.status = WeightedFlowLogMappingStatus::Archived;
        assert!(
            item.move_to(WeightedFlowLogMappingStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
