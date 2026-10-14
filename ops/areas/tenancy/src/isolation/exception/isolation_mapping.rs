use super::*;

use acme_kit::text;

/// Where a isolation mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IsolationMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for IsolationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation mapping as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMapping {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub quantity: i64,
    pub key_id: String,
    pub status: IsolationMappingStatus,
}

impl IsolationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationMappingStatus) -> bool {
        match self.status {
            IsolationMappingStatus::Idle => {
                match next {
                    IsolationMappingStatus::Warming | IsolationMappingStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationMappingStatus::Warming => {
                return next == IsolationMappingStatus::Draining || next == IsolationMappingStatus::Stopped;
            }
            IsolationMappingStatus::Ready => {
                match next {
                    IsolationMappingStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationMappingStatus::Draining => {
                match next {
                    IsolationMappingStatus::Stopped => {
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

    pub fn move_to(&mut self, next: IsolationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.revision as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationMapping {
        IsolationMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            quantity: self.quantity,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a isolation mapping in status idle.
pub fn new_isolation_mapping(code: &str, label: &str) -> IsolationMapping {
    IsolationMapping {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        quantity: 0,
        key_id: "".to_string(),
        status: IsolationMappingStatus::Idle,
    }
}
