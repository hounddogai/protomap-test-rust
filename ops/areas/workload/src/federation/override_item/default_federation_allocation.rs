use super::*;

use acme_kit::text;

/// Where a default federation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFederationAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultFederationAllocationStatus {
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

impl std::fmt::Display for DefaultFederationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default federation allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAllocation {
    pub code: String,
    pub name: String,
    pub description: String,
    pub cost: f64,
    pub segment: String,
    pub sequence: i64,
    pub scope: String,
    pub active: bool,
    pub maximum: f64,
    pub revision: i64,
    pub burst: i64,
    pub status: DefaultFederationAllocationStatus,
}

impl DefaultFederationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultFederationAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultFederationAllocationStatus) -> bool {
        match self.status {
            DefaultFederationAllocationStatus::New => {
                match next {
                    DefaultFederationAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFederationAllocationStatus::Queued => {
                return next == DefaultFederationAllocationStatus::Done
                    || next == DefaultFederationAllocationStatus::Failed;
            }
            DefaultFederationAllocationStatus::Running => {
                match next {
                    DefaultFederationAllocationStatus::Done | DefaultFederationAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFederationAllocationStatus::Done => {
                match next {
                    DefaultFederationAllocationStatus::Failed => {
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

    pub fn move_to(&mut self, next: DefaultFederationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default federation allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.burst as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultFederationAllocation {
        DefaultFederationAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            cost: self.cost,
            segment: self.segment.clone(),
            sequence: self.sequence,
            scope: self.scope.clone(),
            active: self.active,
            maximum: self.maximum,
            revision: self.revision,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a default federation allocation in status new.
pub fn new_default_federation_allocation(code: &str, name: &str) -> DefaultFederationAllocation {
    DefaultFederationAllocation {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        cost: 0.0,
        segment: "".to_string(),
        sequence: 0,
        scope: "".to_string(),
        active: false,
        maximum: 0.0,
        revision: 0,
        burst: 0,
        status: DefaultFederationAllocationStatus::New,
    }
}
