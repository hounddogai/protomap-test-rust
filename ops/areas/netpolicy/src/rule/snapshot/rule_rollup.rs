use super::*;

use acme_kit::text;

/// Where a rule rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RuleRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RuleRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rule rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleRollup {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub key_id: String,
    pub duration_minutes: i64,
    pub priority: i64,
    pub country: String,
    pub status: RuleRollupStatus,
}

impl RuleRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleRollupStatus) -> bool {
        match self.status {
            RuleRollupStatus::Created => {
                match next {
                    RuleRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleRollupStatus::Validated => {
                match next {
                    RuleRollupStatus::Published | RuleRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleRollupStatus::Published => {
                match next {
                    RuleRollupStatus::Retired => {
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

    pub fn move_to(&mut self, next: RuleRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule rollup", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.priority as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleRollup {
        RuleRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            key_id: self.key_id.clone(),
            duration_minutes: self.duration_minutes,
            priority: self.priority,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rule rollup in status created.
pub fn new_rule_rollup(code: &str, name: &str) -> RuleRollup {
    RuleRollup {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        key_id: "".to_string(),
        duration_minutes: 0,
        priority: 0,
        country: "".to_string(),
        status: RuleRollupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rule_rollup("code-213", "name-916");
        assert_eq!(item.status, RuleRollupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_rule_rollup("code-213", "name-916");
        if item.move_to(RuleRollupStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleRollupStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rule_rollup("code-213", "name-916");
        item.status = RuleRollupStatus::Retired;
        assert!(
            item.move_to(RuleRollupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
