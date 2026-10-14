use super::*;

use acme_kit::text;

/// Where a sink tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SinkTierStatus {
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

impl std::fmt::Display for SinkTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkTier {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub visible: bool,
    pub ratio: f64,
    pub budget: f64,
    pub limit: i64,
    pub revision: i64,
    pub status: SinkTierStatus,
}

impl SinkTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkTierStatus) -> bool {
        match self.status {
            SinkTierStatus::Planned => {
                return next == SinkTierStatus::Scheduled || next == SinkTierStatus::InProgress;
            }
            SinkTierStatus::Scheduled => {
                return next == SinkTierStatus::InProgress || next == SinkTierStatus::Cancelled;
            }
            SinkTierStatus::InProgress => {
                match next {
                    SinkTierStatus::Completed | SinkTierStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkTierStatus::Completed => {
                return next == SinkTierStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "sink tier", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ratio - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkTier {
        SinkTier {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            visible: self.visible,
            ratio: self.ratio,
            budget: self.budget,
            limit: self.limit,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a sink tier in status planned.
pub fn new_sink_tier(code: &str, name: &str) -> SinkTier {
    SinkTier {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        visible: false,
        ratio: 0.0,
        budget: 0.0,
        limit: 0,
        revision: 0,
        status: SinkTierStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_sink_tier("code-601", "name-3");
        assert_eq!(item.status, SinkTierStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_sink_tier("code-601", "name-3");
        if item.move_to(SinkTierStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkTierStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_sink_tier("code-601", "name-3");
        item.status = SinkTierStatus::Cancelled;
        assert!(item.move_to(SinkTierStatus::Planned).is_err(), "expected move to fail");
    }
}
