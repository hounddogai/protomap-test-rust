use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched health check assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedHealthCheckAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedHealthCheckAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedHealthCheckAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched health check assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckAssignment {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub grace_days: i64,
    pub revision: i64,
    pub port: i64,
    pub weight: f64,
    pub amount: f64,
    pub attributes: BTreeMap<String, String>,
    pub reviewer: String,
    pub limit: i64,
    pub status: BatchedHealthCheckAssignmentStatus,
}

impl BatchedHealthCheckAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedHealthCheckAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedHealthCheckAssignmentStatus) -> bool {
        match self.status {
            BatchedHealthCheckAssignmentStatus::Created => {
                return next == BatchedHealthCheckAssignmentStatus::Validated
                    || next == BatchedHealthCheckAssignmentStatus::Retired;
            }
            BatchedHealthCheckAssignmentStatus::Validated => {
                return next == BatchedHealthCheckAssignmentStatus::Published
                    || next == BatchedHealthCheckAssignmentStatus::Retired;
            }
            BatchedHealthCheckAssignmentStatus::Published => {
                return next == BatchedHealthCheckAssignmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedHealthCheckAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched health check assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched health check assignment", self.code)
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

    pub fn projected_volume(&self) -> f64 {
        ((self.retries as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedHealthCheckAssignment {
        BatchedHealthCheckAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            grace_days: self.grace_days,
            revision: self.revision,
            port: self.port,
            weight: self.weight,
            amount: self.amount,
            attributes: self.attributes.clone(),
            reviewer: self.reviewer.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched health check assignment in status created.
pub fn new_batched_health_check_assignment(code: &str, name: &str) -> BatchedHealthCheckAssignment {
    BatchedHealthCheckAssignment {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        grace_days: 0,
        revision: 0,
        port: 0,
        weight: 0.0,
        amount: 0.0,
        attributes: BTreeMap::new(),
        reviewer: "".to_string(),
        limit: 0,
        status: BatchedHealthCheckAssignmentStatus::Created,
    }
}
