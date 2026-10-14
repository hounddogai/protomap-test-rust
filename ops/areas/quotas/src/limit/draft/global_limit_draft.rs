use super::*;

use acme_kit::text;

/// Where a global limit draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalLimitDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalLimitDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalLimitDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global limit draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDraft {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub scope: String,
    pub timezone: String,
    pub approved: bool,
    pub status: GlobalLimitDraftStatus,
}

impl GlobalLimitDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalLimitDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalLimitDraftStatus) -> bool {
        match self.status {
            GlobalLimitDraftStatus::Created => {
                return next == GlobalLimitDraftStatus::Retired;
            }
            GlobalLimitDraftStatus::Validated => {
                return next == GlobalLimitDraftStatus::Retired;
            }
            GlobalLimitDraftStatus::Published => {
                match next {
                    GlobalLimitDraftStatus::Retired => {
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

    pub fn move_to(&mut self, next: GlobalLimitDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global limit draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global limit draft", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> GlobalLimitDraft {
        GlobalLimitDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            scope: self.scope.clone(),
            timezone: self.timezone.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a global limit draft in status created.
pub fn new_global_limit_draft(code: &str, title: &str) -> GlobalLimitDraft {
    GlobalLimitDraft {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        scope: "".to_string(),
        timezone: "".to_string(),
        approved: false,
        status: GlobalLimitDraftStatus::Created,
    }
}
