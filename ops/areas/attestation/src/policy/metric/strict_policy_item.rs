use super::*;

use acme_kit::text;

/// Where a strict policy item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPolicyItemStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictPolicyItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for StrictPolicyItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict policy item, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyItem {
    pub code: String,
    pub name: String,
    pub source: String,
    pub summary: String,
    pub weight: f64,
    pub limit: i64,
    pub scope: String,
    pub budget: f64,
    pub burst: i64,
    pub revision: i64,
    pub status: StrictPolicyItemStatus,
}

impl StrictPolicyItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPolicyItemStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPolicyItemStatus) -> bool {
        match self.status {
            StrictPolicyItemStatus::New => {
                return next == StrictPolicyItemStatus::Done;
            }
            StrictPolicyItemStatus::Queued => {
                return next == StrictPolicyItemStatus::Done;
            }
            StrictPolicyItemStatus::Running => {
                match next {
                    StrictPolicyItemStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPolicyItemStatus::Done => {
                match next {
                    StrictPolicyItemStatus::Failed => {
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

    pub fn move_to(&mut self, next: StrictPolicyItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict policy item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.budget + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPolicyItem {
        StrictPolicyItem {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            summary: self.summary.clone(),
            weight: self.weight,
            limit: self.limit,
            scope: self.scope.clone(),
            budget: self.budget,
            burst: self.burst,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a strict policy item in status new.
pub fn new_strict_policy_item(code: &str, name: &str) -> StrictPolicyItem {
    StrictPolicyItem {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        summary: "".to_string(),
        weight: 0.0,
        limit: 0,
        scope: "".to_string(),
        budget: 0.0,
        burst: 0,
        revision: 0,
        status: StrictPolicyItemStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_policy_item("code-816", "name-66");
        assert_eq!(item.status, StrictPolicyItemStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_strict_policy_item("code-816", "name-66");
        if item.move_to(StrictPolicyItemStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictPolicyItemStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_policy_item("code-816", "name-66");
        item.status = StrictPolicyItemStatus::Failed;
        assert!(
            item.move_to(StrictPolicyItemStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
