use super::*;

use acme_kit::text;

/// Where a fallback postmortem entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPostmortemEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackPostmortemEntryStatus {
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

impl std::fmt::Display for FallbackPostmortemEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback postmortem entry as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemEntry {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub window_days: i64,
    pub amount: f64,
    pub description: String,
    pub status: FallbackPostmortemEntryStatus,
}

impl FallbackPostmortemEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPostmortemEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPostmortemEntryStatus) -> bool {
        match self.status {
            FallbackPostmortemEntryStatus::New => {
                match next {
                    FallbackPostmortemEntryStatus::Queued | FallbackPostmortemEntryStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPostmortemEntryStatus::Queued => {
                return next == FallbackPostmortemEntryStatus::Done;
            }
            FallbackPostmortemEntryStatus::Running => {
                return next == FallbackPostmortemEntryStatus::Failed;
            }
            FallbackPostmortemEntryStatus::Done => {
                match next {
                    FallbackPostmortemEntryStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackPostmortemEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback postmortem entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback postmortem entry", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.window_days as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPostmortemEntry {
        FallbackPostmortemEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            window_days: self.window_days,
            amount: self.amount,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback postmortem entry in status new.
pub fn new_fallback_postmortem_entry(code: &str, label: &str) -> FallbackPostmortemEntry {
    FallbackPostmortemEntry {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        window_days: 0,
        amount: 0.0,
        description: "".to_string(),
        status: FallbackPostmortemEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_postmortem_entry("code-318", "label-304");
        assert_eq!(item.status, FallbackPostmortemEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_fallback_postmortem_entry("code-318", "label-304");
        if item.move_to(FallbackPostmortemEntryStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPostmortemEntryStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_postmortem_entry("code-318", "label-304");
        item.status = FallbackPostmortemEntryStatus::Failed;
        assert!(
            item.move_to(FallbackPostmortemEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
