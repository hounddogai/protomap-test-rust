use super::*;

use acme_kit::text;

/// Where a binding cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BindingCycleStatus {
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

impl std::fmt::Display for BindingCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingCycle {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub notes: String,
    pub category: String,
    pub margin: f64,
    pub quota_units: i64,
    pub batch_size: i64,
    pub status: BindingCycleStatus,
}

impl BindingCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingCycleStatus) -> bool {
        match self.status {
            BindingCycleStatus::New => {
                return next == BindingCycleStatus::Running;
            }
            BindingCycleStatus::Queued => {
                return next == BindingCycleStatus::Done || next == BindingCycleStatus::Failed;
            }
            BindingCycleStatus::Running => {
                match next {
                    BindingCycleStatus::Done | BindingCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingCycleStatus::Done => {
                match next {
                    BindingCycleStatus::Failed => {
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

    pub fn move_to(&mut self, next: BindingCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.quota_units as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingCycle {
        BindingCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            notes: self.notes.clone(),
            category: self.category.clone(),
            margin: self.margin,
            quota_units: self.quota_units,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a binding cycle in status new.
pub fn new_binding_cycle(code: &str, label: &str) -> BindingCycle {
    BindingCycle {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        notes: "".to_string(),
        category: "".to_string(),
        margin: 0.0,
        quota_units: 0,
        batch_size: 0,
        status: BindingCycleStatus::New,
    }
}
