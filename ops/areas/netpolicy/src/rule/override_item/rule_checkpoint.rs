use super::*;

use acme_kit::text;

/// Where a rule checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RuleCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RuleCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleCheckpoint {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub namespace: String,
    pub retries: i64,
    pub threshold: f64,
    pub source: String,
    pub scope: String,
    pub status: RuleCheckpointStatus,
}

impl RuleCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleCheckpointStatus) -> bool {
        match self.status {
            RuleCheckpointStatus::Created => {
                match next {
                    RuleCheckpointStatus::Validated | RuleCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleCheckpointStatus::Validated => {
                match next {
                    RuleCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleCheckpointStatus::Published => {
                return next == RuleCheckpointStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.retries as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleCheckpoint {
        RuleCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            namespace: self.namespace.clone(),
            retries: self.retries,
            threshold: self.threshold,
            source: self.source.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rule checkpoint in status created.
pub fn new_rule_checkpoint(code: &str, title: &str) -> RuleCheckpoint {
    RuleCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        namespace: "".to_string(),
        retries: 0,
        threshold: 0.0,
        source: "".to_string(),
        scope: "".to_string(),
        status: RuleCheckpointStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rule_checkpoint("code-487", "title-91");
        assert_eq!(item.status, RuleCheckpointStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_rule_checkpoint("code-487", "title-91");
        if item.move_to(RuleCheckpointStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleCheckpointStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rule_checkpoint("code-487", "title-91");
        item.status = RuleCheckpointStatus::Retired;
        assert!(
            item.move_to(RuleCheckpointStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
