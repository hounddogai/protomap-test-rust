use super::*;

use acme_kit::text;

/// Where a default plan rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPlanRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultPlanRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultPlanRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default plan rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanRollup {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub source: String,
    pub quantity: i64,
    pub region: String,
    pub archived: bool,
    pub revision: i64,
    pub status: DefaultPlanRollupStatus,
}

impl DefaultPlanRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPlanRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPlanRollupStatus) -> bool {
        match self.status {
            DefaultPlanRollupStatus::Proposed => {
                match next {
                    DefaultPlanRollupStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPlanRollupStatus::Approved => {
                return next == DefaultPlanRollupStatus::Withdrawn;
            }
            DefaultPlanRollupStatus::Rejected => {
                return next == DefaultPlanRollupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultPlanRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default plan rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.revision as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultPlanRollup {
        DefaultPlanRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            source: self.source.clone(),
            quantity: self.quantity,
            region: self.region.clone(),
            archived: self.archived,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default plan rollup in status proposed.
pub fn new_default_plan_rollup(code: &str, name: &str) -> DefaultPlanRollup {
    DefaultPlanRollup {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        source: "".to_string(),
        quantity: 0,
        region: "".to_string(),
        archived: false,
        revision: 0,
        status: DefaultPlanRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_plan_rollup("code-540", "name-28");
        assert_eq!(item.status, DefaultPlanRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_default_plan_rollup("code-540", "name-28");
        if item.move_to(DefaultPlanRollupStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultPlanRollupStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_plan_rollup("code-540", "name-28");
        item.status = DefaultPlanRollupStatus::Withdrawn;
        assert!(
            item.move_to(DefaultPlanRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
