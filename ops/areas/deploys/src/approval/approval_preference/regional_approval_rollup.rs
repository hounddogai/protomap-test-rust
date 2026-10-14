use super::*;

use acme_kit::text;

/// Where a regional approval rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalApprovalRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalApprovalRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalApprovalRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional approval rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalRollup {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub visible: bool,
    pub name: String,
    pub weight: f64,
    pub ttl_seconds: i64,
    pub channel: String,
    pub notes: String,
    pub ceiling: f64,
    pub timezone: String,
    pub status: RegionalApprovalRollupStatus,
}

impl RegionalApprovalRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalApprovalRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalApprovalRollupStatus) -> bool {
        match self.status {
            RegionalApprovalRollupStatus::Proposed => {
                return next == RegionalApprovalRollupStatus::Withdrawn;
            }
            RegionalApprovalRollupStatus::Approved => {
                return next == RegionalApprovalRollupStatus::Rejected;
            }
            RegionalApprovalRollupStatus::Rejected => {
                match next {
                    RegionalApprovalRollupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionalApprovalRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional approval rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.weight + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalApprovalRollup {
        RegionalApprovalRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            visible: self.visible,
            name: self.name.clone(),
            weight: self.weight,
            ttl_seconds: self.ttl_seconds,
            channel: self.channel.clone(),
            notes: self.notes.clone(),
            ceiling: self.ceiling,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a regional approval rollup in status proposed.
pub fn new_regional_approval_rollup(code: &str, label: &str) -> RegionalApprovalRollup {
    RegionalApprovalRollup {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        visible: false,
        name: "".to_string(),
        weight: 0.0,
        ttl_seconds: 0,
        channel: "".to_string(),
        notes: "".to_string(),
        ceiling: 0.0,
        timezone: "".to_string(),
        status: RegionalApprovalRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_approval_rollup("code-38", "label-891");
        assert_eq!(item.status, RegionalApprovalRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_regional_approval_rollup("code-38", "label-891");
        if item.move_to(RegionalApprovalRollupStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalApprovalRollupStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_approval_rollup("code-38", "label-891");
        item.status = RegionalApprovalRollupStatus::Withdrawn;
        assert!(
            item.move_to(RegionalApprovalRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
