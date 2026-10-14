use super::*;

use acme_kit::text;

/// Where a strict rule draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRuleDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictRuleDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for StrictRuleDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict rule draft as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleDraft {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub cost: f64,
    pub floor_value: f64,
    pub channel: String,
    pub weight: f64,
    pub updated_at: i64,
    pub scope: String,
    pub notes: String,
    pub status: StrictRuleDraftStatus,
}

impl StrictRuleDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRuleDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRuleDraftStatus) -> bool {
        match self.status {
            StrictRuleDraftStatus::Idle => {
                return next == StrictRuleDraftStatus::Stopped;
            }
            StrictRuleDraftStatus::Warming => {
                return next == StrictRuleDraftStatus::Ready || next == StrictRuleDraftStatus::Draining;
            }
            StrictRuleDraftStatus::Ready => {
                match next {
                    StrictRuleDraftStatus::Draining | StrictRuleDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRuleDraftStatus::Draining => {
                match next {
                    StrictRuleDraftStatus::Stopped => {
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

    pub fn move_to(&mut self, next: StrictRuleDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict rule draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.updated_at as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRuleDraft {
        StrictRuleDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            cost: self.cost,
            floor_value: self.floor_value,
            channel: self.channel.clone(),
            weight: self.weight,
            updated_at: self.updated_at,
            scope: self.scope.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a strict rule draft in status idle.
pub fn new_strict_rule_draft(code: &str, title: &str) -> StrictRuleDraft {
    StrictRuleDraft {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        cost: 0.0,
        floor_value: 0.0,
        channel: "".to_string(),
        weight: 0.0,
        updated_at: 0,
        scope: "".to_string(),
        notes: "".to_string(),
        status: StrictRuleDraftStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_rule_draft("code-277", "title-573");
        assert_eq!(item.status, StrictRuleDraftStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_strict_rule_draft("code-277", "title-573");
        if item.move_to(StrictRuleDraftStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRuleDraftStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_rule_draft("code-277", "title-573");
        item.status = StrictRuleDraftStatus::Stopped;
        assert!(
            item.move_to(StrictRuleDraftStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
