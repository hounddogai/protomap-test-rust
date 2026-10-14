use super::*;

use acme_kit::text;

/// Where a legacy failover adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFailoverAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyFailoverAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyFailoverAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy failover adjustment as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverAdjustment {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub ceiling: f64,
    pub region: String,
    pub description: String,
    pub duration_minutes: i64,
    pub maximum: f64,
    pub status: LegacyFailoverAdjustmentStatus,
}

impl LegacyFailoverAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFailoverAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFailoverAdjustmentStatus) -> bool {
        match self.status {
            LegacyFailoverAdjustmentStatus::Proposed => {
                match next {
                    LegacyFailoverAdjustmentStatus::Rejected | LegacyFailoverAdjustmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFailoverAdjustmentStatus::Approved => {
                match next {
                    LegacyFailoverAdjustmentStatus::Rejected | LegacyFailoverAdjustmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFailoverAdjustmentStatus::Rejected => {
                match next {
                    LegacyFailoverAdjustmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LegacyFailoverAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy failover adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        (self.ceiling * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFailoverAdjustment {
        LegacyFailoverAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            ceiling: self.ceiling,
            region: self.region.clone(),
            description: self.description.clone(),
            duration_minutes: self.duration_minutes,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a legacy failover adjustment in status proposed.
pub fn new_legacy_failover_adjustment(code: &str, name: &str) -> LegacyFailoverAdjustment {
    LegacyFailoverAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        ceiling: 0.0,
        region: "".to_string(),
        description: "".to_string(),
        duration_minutes: 0,
        maximum: 0.0,
        status: LegacyFailoverAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_failover_adjustment("code-418", "name-399");
        assert_eq!(item.status, LegacyFailoverAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_legacy_failover_adjustment("code-418", "name-399");
        if item.move_to(LegacyFailoverAdjustmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFailoverAdjustmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_failover_adjustment("code-418", "name-399");
        item.status = LegacyFailoverAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(LegacyFailoverAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
