use super::*;

use acme_kit::text;

/// Where a seasonal rule cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRuleCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalRuleCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for SeasonalRuleCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal rule cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRuleCycle {
    pub code: String,
    pub label: String,
    pub description: String,
    pub ceiling: f64,
    pub revision: i64,
    pub active: bool,
    pub title: String,
    pub expires_at: i64,
    pub attempts: i64,
    pub status: SeasonalRuleCycleStatus,
}

impl SeasonalRuleCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRuleCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRuleCycleStatus) -> bool {
        match self.status {
            SeasonalRuleCycleStatus::New => {
                match next {
                    SeasonalRuleCycleStatus::Done | SeasonalRuleCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRuleCycleStatus::Queued => {
                return next == SeasonalRuleCycleStatus::Failed;
            }
            SeasonalRuleCycleStatus::Running => {
                return next == SeasonalRuleCycleStatus::Failed;
            }
            SeasonalRuleCycleStatus::Done => {
                return next == SeasonalRuleCycleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalRuleCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal rule cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.revision as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRuleCycle {
        SeasonalRuleCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            ceiling: self.ceiling,
            revision: self.revision,
            active: self.active,
            title: self.title.clone(),
            expires_at: self.expires_at,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal rule cycle in status new.
pub fn new_seasonal_rule_cycle(code: &str, label: &str) -> SeasonalRuleCycle {
    SeasonalRuleCycle {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        ceiling: 0.0,
        revision: 0,
        active: false,
        title: "".to_string(),
        expires_at: 0,
        attempts: 0,
        status: SeasonalRuleCycleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_rule_cycle("code-928", "label-681");
        assert_eq!(item.status, SeasonalRuleCycleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_seasonal_rule_cycle("code-928", "label-681");
        if item.move_to(SeasonalRuleCycleStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRuleCycleStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_rule_cycle("code-928", "label-681");
        item.status = SeasonalRuleCycleStatus::Failed;
        assert!(
            item.move_to(SeasonalRuleCycleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
