use super::*;

use acme_kit::text;

/// Where a slo rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SloRuleStatus {
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

impl std::fmt::Display for SloRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One slo rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloRule {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub region: String,
    pub batch_size: i64,
    pub contact_email: String,
    pub active: bool,
    pub status: SloRuleStatus,
}

impl SloRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloRuleStatus) -> bool {
        match self.status {
            SloRuleStatus::Planned => {
                match next {
                    SloRuleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloRuleStatus::Scheduled => {
                return next == SloRuleStatus::InProgress;
            }
            SloRuleStatus::InProgress => {
                return next == SloRuleStatus::Completed;
            }
            SloRuleStatus::Completed => {
                return next == SloRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "slo rule", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> SloRule {
        SloRule {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            region: self.region.clone(),
            batch_size: self.batch_size,
            contact_email: self.contact_email.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a slo rule in status planned.
pub fn new_slo_rule(code: &str, name: &str) -> SloRule {
    SloRule {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        region: "".to_string(),
        batch_size: 0,
        contact_email: "".to_string(),
        active: false,
        status: SloRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_slo_rule("code-27", "name-101");
        assert_eq!(item.status, SloRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_slo_rule("code-27", "name-101");
        if item.move_to(SloRuleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloRuleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_slo_rule("code-27", "name-101");
        item.status = SloRuleStatus::Cancelled;
        assert!(item.move_to(SloRuleStatus::Planned).is_err(), "expected move to fail");
    }
}
