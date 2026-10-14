use super::*;

use acme_kit::text;

/// Where a lenient silence override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSilenceOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientSilenceOverrideStatus {
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

impl std::fmt::Display for LenientSilenceOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient silence override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceOverride {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub limit: i64,
    pub retries: i64,
    pub approved: bool,
    pub ceiling: f64,
    pub status: LenientSilenceOverrideStatus,
}

impl LenientSilenceOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSilenceOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSilenceOverrideStatus) -> bool {
        match self.status {
            LenientSilenceOverrideStatus::New => {
                match next {
                    LenientSilenceOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSilenceOverrideStatus::Queued => {
                match next {
                    LenientSilenceOverrideStatus::Running | LenientSilenceOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSilenceOverrideStatus::Running => {
                return next == LenientSilenceOverrideStatus::Done || next == LenientSilenceOverrideStatus::Failed;
            }
            LenientSilenceOverrideStatus::Done => {
                return next == LenientSilenceOverrideStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientSilenceOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient silence override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.priority as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSilenceOverride {
        LenientSilenceOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            limit: self.limit,
            retries: self.retries,
            approved: self.approved,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient silence override in status new.
pub fn new_lenient_silence_override(code: &str, label: &str) -> LenientSilenceOverride {
    LenientSilenceOverride {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        limit: 0,
        retries: 0,
        approved: false,
        ceiling: 0.0,
        status: LenientSilenceOverrideStatus::New,
    }
}
