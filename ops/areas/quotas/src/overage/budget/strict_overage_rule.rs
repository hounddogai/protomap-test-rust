use super::*;

use acme_kit::text;

/// Where a strict overage rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictOverageRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictOverageRuleStatus {
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

impl std::fmt::Display for StrictOverageRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict overage rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageRule {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub batch_size: i64,
    pub ends_at: i64,
    pub quantity: i64,
    pub channel: String,
    pub checksum: String,
    pub ceiling: f64,
    pub approved: bool,
    pub status: StrictOverageRuleStatus,
}

impl StrictOverageRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictOverageRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictOverageRuleStatus) -> bool {
        match self.status {
            StrictOverageRuleStatus::Planned => {
                return next == StrictOverageRuleStatus::Scheduled || next == StrictOverageRuleStatus::Cancelled;
            }
            StrictOverageRuleStatus::Scheduled => {
                return next == StrictOverageRuleStatus::InProgress;
            }
            StrictOverageRuleStatus::InProgress => {
                match next {
                    StrictOverageRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictOverageRuleStatus::Completed => {
                return next == StrictOverageRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictOverageRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict overage rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.sequence as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictOverageRule {
        StrictOverageRule {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            batch_size: self.batch_size,
            ends_at: self.ends_at,
            quantity: self.quantity,
            channel: self.channel.clone(),
            checksum: self.checksum.clone(),
            ceiling: self.ceiling,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a strict overage rule in status planned.
pub fn new_strict_overage_rule(code: &str, name: &str) -> StrictOverageRule {
    StrictOverageRule {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        batch_size: 0,
        ends_at: 0,
        quantity: 0,
        channel: "".to_string(),
        checksum: "".to_string(),
        ceiling: 0.0,
        approved: false,
        status: StrictOverageRuleStatus::Planned,
    }
}
