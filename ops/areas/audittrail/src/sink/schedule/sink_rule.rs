use super::*;

use acme_kit::text;

/// Where a sink rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SinkRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SinkRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkRule {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub source: String,
    pub approved: bool,
    pub maximum: f64,
    pub description: String,
    pub channel: String,
    pub pinned: bool,
    pub limit: i64,
    pub status: SinkRuleStatus,
}

impl SinkRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkRuleStatus) -> bool {
        match self.status {
            SinkRuleStatus::Proposed => {
                return next == SinkRuleStatus::Approved || next == SinkRuleStatus::Rejected;
            }
            SinkRuleStatus::Approved => {
                match next {
                    SinkRuleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkRuleStatus::Rejected => {
                return next == SinkRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.limit as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkRule {
        SinkRule {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            source: self.source.clone(),
            approved: self.approved,
            maximum: self.maximum,
            description: self.description.clone(),
            channel: self.channel.clone(),
            pinned: self.pinned,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a sink rule in status proposed.
pub fn new_sink_rule(code: &str, name: &str) -> SinkRule {
    SinkRule {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        source: "".to_string(),
        approved: false,
        maximum: 0.0,
        description: "".to_string(),
        channel: "".to_string(),
        pinned: false,
        limit: 0,
        status: SinkRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_sink_rule("code-277", "name-467");
        assert_eq!(item.status, SinkRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_sink_rule("code-277", "name-467");
        if item.move_to(SinkRuleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkRuleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_sink_rule("code-277", "name-467");
        item.status = SinkRuleStatus::Withdrawn;
        assert!(item.move_to(SinkRuleStatus::Proposed).is_err(), "expected move to fail");
    }
}
