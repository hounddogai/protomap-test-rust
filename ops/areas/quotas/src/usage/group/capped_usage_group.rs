use super::*;

use acme_kit::text;

/// Where a capped usage group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedUsageGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedUsageGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedUsageGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped usage group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedUsageGroup {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub sequence: i64,
    pub source: String,
    pub phone: String,
    pub status: CappedUsageGroupStatus,
}

impl CappedUsageGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedUsageGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedUsageGroupStatus) -> bool {
        match self.status {
            CappedUsageGroupStatus::Proposed => {
                match next {
                    CappedUsageGroupStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedUsageGroupStatus::Approved => {
                return next == CappedUsageGroupStatus::Withdrawn;
            }
            CappedUsageGroupStatus::Rejected => {
                return next == CappedUsageGroupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedUsageGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped usage group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped usage group", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.sequence as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedUsageGroup {
        CappedUsageGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            sequence: self.sequence,
            source: self.source.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a capped usage group in status proposed.
pub fn new_capped_usage_group(code: &str, label: &str) -> CappedUsageGroup {
    CappedUsageGroup {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        sequence: 0,
        source: "".to_string(),
        phone: "".to_string(),
        status: CappedUsageGroupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_usage_group("code-690", "label-47");
        assert_eq!(item.status, CappedUsageGroupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_capped_usage_group("code-690", "label-47");
        if item.move_to(CappedUsageGroupStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedUsageGroupStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_usage_group("code-690", "label-47");
        item.status = CappedUsageGroupStatus::Withdrawn;
        assert!(
            item.move_to(CappedUsageGroupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
