use super::*;

use acme_kit::text;

/// Where a default grant ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultGrantLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultGrantLedgerStatus {
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

impl std::fmt::Display for DefaultGrantLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default grant ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantLedger {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub cluster: String,
    pub ceiling: f64,
    pub label: String,
    pub tags: Vec<String>,
    pub status: DefaultGrantLedgerStatus,
}

impl DefaultGrantLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultGrantLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultGrantLedgerStatus) -> bool {
        match self.status {
            DefaultGrantLedgerStatus::New => {
                return next == DefaultGrantLedgerStatus::Running || next == DefaultGrantLedgerStatus::Done;
            }
            DefaultGrantLedgerStatus::Queued => {
                return next == DefaultGrantLedgerStatus::Done || next == DefaultGrantLedgerStatus::Failed;
            }
            DefaultGrantLedgerStatus::Running => {
                match next {
                    DefaultGrantLedgerStatus::Done | DefaultGrantLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultGrantLedgerStatus::Done => {
                match next {
                    DefaultGrantLedgerStatus::Failed => {
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

    pub fn move_to(&mut self, next: DefaultGrantLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default grant ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ratio - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultGrantLedger {
        DefaultGrantLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            cluster: self.cluster.clone(),
            ceiling: self.ceiling,
            label: self.label.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a default grant ledger in status new.
pub fn new_default_grant_ledger(code: &str, name: &str) -> DefaultGrantLedger {
    DefaultGrantLedger {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        cluster: "".to_string(),
        ceiling: 0.0,
        label: "".to_string(),
        tags: Vec::new(),
        status: DefaultGrantLedgerStatus::New,
    }
}
