use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a failover assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FailoverAssignmentStatus {
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

impl std::fmt::Display for FailoverAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover assignment as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAssignment {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub cluster: String,
    pub threshold: f64,
    pub rate: f64,
    pub issuer: String,
    pub ceiling: f64,
    pub currency: String,
    pub attributes: BTreeMap<String, String>,
    pub revision: i64,
    pub status: FailoverAssignmentStatus,
}

impl FailoverAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverAssignmentStatus) -> bool {
        match self.status {
            FailoverAssignmentStatus::New => {
                return next == FailoverAssignmentStatus::Done;
            }
            FailoverAssignmentStatus::Queued => {
                return next == FailoverAssignmentStatus::Running || next == FailoverAssignmentStatus::Done;
            }
            FailoverAssignmentStatus::Running => {
                match next {
                    FailoverAssignmentStatus::Done | FailoverAssignmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverAssignmentStatus::Done => {
                return next == FailoverAssignmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "failover assignment", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_total(&self) -> f64 {
        ((self.threshold - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverAssignment {
        FailoverAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            cluster: self.cluster.clone(),
            threshold: self.threshold,
            rate: self.rate,
            issuer: self.issuer.clone(),
            ceiling: self.ceiling,
            currency: self.currency.clone(),
            attributes: self.attributes.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a failover assignment in status new.
pub fn new_failover_assignment(code: &str, name: &str) -> FailoverAssignment {
    FailoverAssignment {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        cluster: "".to_string(),
        threshold: 0.0,
        rate: 0.0,
        issuer: "".to_string(),
        ceiling: 0.0,
        currency: "".to_string(),
        attributes: BTreeMap::new(),
        revision: 0,
        status: FailoverAssignmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_failover_assignment("code-11", "name-18");
        assert_eq!(item.status, FailoverAssignmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_failover_assignment("code-11", "name-18");
        if item.move_to(FailoverAssignmentStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverAssignmentStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_failover_assignment("code-11", "name-18");
        item.status = FailoverAssignmentStatus::Failed;
        assert!(
            item.move_to(FailoverAssignmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
