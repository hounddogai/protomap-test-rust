use super::*;

use acme_kit::text;

/// Where a failover target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FailoverTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for FailoverTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover target as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverTarget {
    pub code: String,
    pub label: String,
    pub host: String,
    pub source: String,
    pub revision: i64,
    pub unit: String,
    pub status: FailoverTargetStatus,
}

impl FailoverTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverTargetStatus) -> bool {
        match self.status {
            FailoverTargetStatus::Draft => {
                match next {
                    FailoverTargetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverTargetStatus::Pending => {
                return next == FailoverTargetStatus::Suspended || next == FailoverTargetStatus::Closed;
            }
            FailoverTargetStatus::Active => {
                match next {
                    FailoverTargetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverTargetStatus::Suspended => {
                match next {
                    FailoverTargetStatus::Closed => {
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

    pub fn move_to(&mut self, next: FailoverTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> FailoverTarget {
        FailoverTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            source: self.source.clone(),
            revision: self.revision,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a failover target in status draft.
pub fn new_failover_target(code: &str, label: &str) -> FailoverTarget {
    FailoverTarget {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        source: "".to_string(),
        revision: 0,
        unit: "".to_string(),
        status: FailoverTargetStatus::Draft,
    }
}
