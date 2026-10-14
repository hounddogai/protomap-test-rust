use super::*;

use acme_kit::text;

/// Where a regional slo override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSloOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalSloOverrideStatus {
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

impl std::fmt::Display for RegionalSloOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional slo override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloOverride {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub quantity: i64,
    pub revision: i64,
    pub approved: bool,
    pub status: RegionalSloOverrideStatus,
}

impl RegionalSloOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSloOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSloOverrideStatus) -> bool {
        match self.status {
            RegionalSloOverrideStatus::Planned => {
                return next == RegionalSloOverrideStatus::Scheduled || next == RegionalSloOverrideStatus::Completed;
            }
            RegionalSloOverrideStatus::Scheduled => {
                return next == RegionalSloOverrideStatus::InProgress;
            }
            RegionalSloOverrideStatus::InProgress => {
                return next == RegionalSloOverrideStatus::Cancelled;
            }
            RegionalSloOverrideStatus::Completed => {
                return next == RegionalSloOverrideStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalSloOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional slo override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.revision as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSloOverride {
        RegionalSloOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            quantity: self.quantity,
            revision: self.revision,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a regional slo override in status planned.
pub fn new_regional_slo_override(code: &str, name: &str) -> RegionalSloOverride {
    RegionalSloOverride {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        quantity: 0,
        revision: 0,
        approved: false,
        status: RegionalSloOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_regional_slo_override("code-97", "name-288");
        assert_eq!(item.status, RegionalSloOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_regional_slo_override("code-97", "name-288");
        if item.move_to(RegionalSloOverrideStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalSloOverrideStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_regional_slo_override("code-97", "name-288");
        item.status = RegionalSloOverrideStatus::Cancelled;
        assert!(
            item.move_to(RegionalSloOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
