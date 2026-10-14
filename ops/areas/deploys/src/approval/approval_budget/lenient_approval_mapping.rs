use super::*;

use acme_kit::text;

/// Where a lenient approval mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientApprovalMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientApprovalMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LenientApprovalMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient approval mapping as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalMapping {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub sequence: i64,
    pub ratio: f64,
    pub margin: f64,
    pub key_id: String,
    pub status: LenientApprovalMappingStatus,
}

impl LenientApprovalMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientApprovalMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientApprovalMappingStatus) -> bool {
        match self.status {
            LenientApprovalMappingStatus::Planned => {
                return next == LenientApprovalMappingStatus::InProgress;
            }
            LenientApprovalMappingStatus::Scheduled => {
                match next {
                    LenientApprovalMappingStatus::InProgress | LenientApprovalMappingStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientApprovalMappingStatus::InProgress => {
                return next == LenientApprovalMappingStatus::Completed
                    || next == LenientApprovalMappingStatus::Cancelled;
            }
            LenientApprovalMappingStatus::Completed => {
                match next {
                    LenientApprovalMappingStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientApprovalMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient approval mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ratio - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientApprovalMapping {
        LenientApprovalMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            sequence: self.sequence,
            ratio: self.ratio,
            margin: self.margin,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient approval mapping in status planned.
pub fn new_lenient_approval_mapping(code: &str, label: &str) -> LenientApprovalMapping {
    LenientApprovalMapping {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        sequence: 0,
        ratio: 0.0,
        margin: 0.0,
        key_id: "".to_string(),
        status: LenientApprovalMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_approval_mapping("code-50", "label-544");
        assert_eq!(item.status, LenientApprovalMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_lenient_approval_mapping("code-50", "label-544");
        if item.move_to(LenientApprovalMappingStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientApprovalMappingStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_approval_mapping("code-50", "label-544");
        item.status = LenientApprovalMappingStatus::Cancelled;
        assert!(
            item.move_to(LenientApprovalMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
