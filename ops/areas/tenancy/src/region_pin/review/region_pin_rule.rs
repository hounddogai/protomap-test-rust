use super::*;

use acme_kit::text;

/// Where a region pin rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionPinRuleStatus {
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

impl std::fmt::Display for RegionPinRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One region pin rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRule {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub floor_value: f64,
    pub region: String,
    pub limit: i64,
    pub quota_units: i64,
    pub tags: Vec<String>,
    pub status: RegionPinRuleStatus,
}

impl RegionPinRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinRuleStatus) -> bool {
        match self.status {
            RegionPinRuleStatus::Planned => {
                return next == RegionPinRuleStatus::Scheduled || next == RegionPinRuleStatus::InProgress;
            }
            RegionPinRuleStatus::Scheduled => {
                match next {
                    RegionPinRuleStatus::InProgress | RegionPinRuleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinRuleStatus::InProgress => {
                return next == RegionPinRuleStatus::Completed || next == RegionPinRuleStatus::Cancelled;
            }
            RegionPinRuleStatus::Completed => {
                match next {
                    RegionPinRuleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionPinRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.quota_units as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinRule {
        RegionPinRule {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            floor_value: self.floor_value,
            region: self.region.clone(),
            limit: self.limit,
            quota_units: self.quota_units,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a region pin rule in status planned.
pub fn new_region_pin_rule(code: &str, name: &str) -> RegionPinRule {
    RegionPinRule {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        floor_value: 0.0,
        region: "".to_string(),
        limit: 0,
        quota_units: 0,
        tags: Vec::new(),
        status: RegionPinRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_region_pin_rule("code-582", "name-723");
        assert_eq!(item.status, RegionPinRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_region_pin_rule("code-582", "name-723");
        if item.move_to(RegionPinRuleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinRuleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_region_pin_rule("code-582", "name-723");
        item.status = RegionPinRuleStatus::Cancelled;
        assert!(
            item.move_to(RegionPinRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
