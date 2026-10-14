use super::*;

use acme_kit::text;

/// Where a allowlist target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AllowlistTargetStatus {
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

impl std::fmt::Display for AllowlistTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTarget {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub currency: String,
    pub port: i64,
    pub minimum: f64,
    pub tags: Vec<String>,
    pub active: bool,
    pub status: AllowlistTargetStatus,
}

impl AllowlistTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistTargetStatus) -> bool {
        match self.status {
            AllowlistTargetStatus::New => {
                return next == AllowlistTargetStatus::Done;
            }
            AllowlistTargetStatus::Queued => {
                return next == AllowlistTargetStatus::Running || next == AllowlistTargetStatus::Done;
            }
            AllowlistTargetStatus::Running => {
                match next {
                    AllowlistTargetStatus::Done | AllowlistTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistTargetStatus::Done => {
                match next {
                    AllowlistTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: AllowlistTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "allowlist target", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.port as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistTarget {
        AllowlistTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            currency: self.currency.clone(),
            port: self.port,
            minimum: self.minimum,
            tags: self.tags.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a allowlist target in status new.
pub fn new_allowlist_target(code: &str, label: &str) -> AllowlistTarget {
    AllowlistTarget {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        currency: "".to_string(),
        port: 0,
        minimum: 0.0,
        tags: Vec::new(),
        active: false,
        status: AllowlistTargetStatus::New,
    }
}
