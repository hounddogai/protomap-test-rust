use super::*;

use acme_kit::text;

/// Where a retention allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionAllocationStatus {
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

impl std::fmt::Display for RetentionAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAllocation {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub country: String,
    pub source: String,
    pub namespace: String,
    pub margin: f64,
    pub locale: String,
    pub burst: i64,
    pub status: RetentionAllocationStatus,
}

impl RetentionAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionAllocationStatus) -> bool {
        match self.status {
            RetentionAllocationStatus::New => {
                match next {
                    RetentionAllocationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAllocationStatus::Queued => {
                match next {
                    RetentionAllocationStatus::Running | RetentionAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAllocationStatus::Running => {
                match next {
                    RetentionAllocationStatus::Done | RetentionAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAllocationStatus::Done => {
                return next == RetentionAllocationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.margin - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionAllocation {
        RetentionAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            country: self.country.clone(),
            source: self.source.clone(),
            namespace: self.namespace.clone(),
            margin: self.margin,
            locale: self.locale.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a retention allocation in status new.
pub fn new_retention_allocation(code: &str, name: &str) -> RetentionAllocation {
    RetentionAllocation {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        country: "".to_string(),
        source: "".to_string(),
        namespace: "".to_string(),
        margin: 0.0,
        locale: "".to_string(),
        burst: 0,
        status: RetentionAllocationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_retention_allocation("code-289", "name-975");
        assert_eq!(item.status, RetentionAllocationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_retention_allocation("code-289", "name-975");
        if item.move_to(RetentionAllocationStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionAllocationStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_retention_allocation("code-289", "name-975");
        item.status = RetentionAllocationStatus::Failed;
        assert!(
            item.move_to(RetentionAllocationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
