use super::*;

use acme_kit::text;

/// Where a lenient flow log entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFlowLogEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientFlowLogEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LenientFlowLogEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient flow log entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEntry {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub window_days: i64,
    pub limit: i64,
    pub retries: i64,
    pub status: LenientFlowLogEntryStatus,
}

impl LenientFlowLogEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFlowLogEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFlowLogEntryStatus) -> bool {
        match self.status {
            LenientFlowLogEntryStatus::Open => {
                return next == LenientFlowLogEntryStatus::InReview || next == LenientFlowLogEntryStatus::Archived;
            }
            LenientFlowLogEntryStatus::InReview => {
                return next == LenientFlowLogEntryStatus::Archived;
            }
            LenientFlowLogEntryStatus::Resolved => {
                return next == LenientFlowLogEntryStatus::Reopened;
            }
            LenientFlowLogEntryStatus::Reopened => {
                match next {
                    LenientFlowLogEntryStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientFlowLogEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient flow log entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.window_days as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientFlowLogEntry {
        LenientFlowLogEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            window_days: self.window_days,
            limit: self.limit,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient flow log entry in status open.
pub fn new_lenient_flow_log_entry(code: &str, label: &str) -> LenientFlowLogEntry {
    LenientFlowLogEntry {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        window_days: 0,
        limit: 0,
        retries: 0,
        status: LenientFlowLogEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_flow_log_entry("code-380", "label-231");
        assert_eq!(item.status, LenientFlowLogEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_lenient_flow_log_entry("code-380", "label-231");
        if item.move_to(LenientFlowLogEntryStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFlowLogEntryStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_flow_log_entry("code-380", "label-231");
        item.status = LenientFlowLogEntryStatus::Archived;
        assert!(
            item.move_to(LenientFlowLogEntryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
