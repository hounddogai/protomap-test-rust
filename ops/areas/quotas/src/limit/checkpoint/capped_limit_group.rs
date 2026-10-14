use super::*;

use acme_kit::text;

/// Where a capped limit group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedLimitGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedLimitGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedLimitGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped limit group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedLimitGroup {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub margin: f64,
    pub window_days: i64,
    pub weight: f64,
    pub status: CappedLimitGroupStatus,
}

impl CappedLimitGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedLimitGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedLimitGroupStatus) -> bool {
        match self.status {
            CappedLimitGroupStatus::Proposed => {
                match next {
                    CappedLimitGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedLimitGroupStatus::Approved => {
                return next == CappedLimitGroupStatus::Rejected || next == CappedLimitGroupStatus::Withdrawn;
            }
            CappedLimitGroupStatus::Rejected => {
                return next == CappedLimitGroupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedLimitGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped limit group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.window_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedLimitGroup {
        CappedLimitGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            margin: self.margin,
            window_days: self.window_days,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped limit group in status proposed.
pub fn new_capped_limit_group(code: &str, label: &str) -> CappedLimitGroup {
    CappedLimitGroup {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        margin: 0.0,
        window_days: 0,
        weight: 0.0,
        status: CappedLimitGroupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_limit_group("code-418", "label-951");
        assert_eq!(item.status, CappedLimitGroupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_capped_limit_group("code-418", "label-951");
        if item.move_to(CappedLimitGroupStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedLimitGroupStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_limit_group("code-418", "label-951");
        item.status = CappedLimitGroupStatus::Withdrawn;
        assert!(
            item.move_to(CappedLimitGroupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
