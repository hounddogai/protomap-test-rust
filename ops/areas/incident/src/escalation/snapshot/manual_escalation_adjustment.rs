use super::*;

use acme_kit::text;

/// Where a manual escalation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualEscalationAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualEscalationAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ManualEscalationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual escalation adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualEscalationAdjustment {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub pinned: bool,
    pub burst: i64,
    pub archived: bool,
    pub floor_value: f64,
    pub rate: f64,
    pub channel: String,
    pub name: String,
    pub status: ManualEscalationAdjustmentStatus,
}

impl ManualEscalationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualEscalationAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualEscalationAdjustmentStatus) -> bool {
        match self.status {
            ManualEscalationAdjustmentStatus::Draft => {
                match next {
                    ManualEscalationAdjustmentStatus::Pending | ManualEscalationAdjustmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEscalationAdjustmentStatus::Pending => {
                match next {
                    ManualEscalationAdjustmentStatus::Active | ManualEscalationAdjustmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEscalationAdjustmentStatus::Active => {
                match next {
                    ManualEscalationAdjustmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEscalationAdjustmentStatus::Suspended => {
                match next {
                    ManualEscalationAdjustmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualEscalationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual escalation adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.floor_value * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualEscalationAdjustment {
        ManualEscalationAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            pinned: self.pinned,
            burst: self.burst,
            archived: self.archived,
            floor_value: self.floor_value,
            rate: self.rate,
            channel: self.channel.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a manual escalation adjustment in status draft.
pub fn new_manual_escalation_adjustment(code: &str, title: &str) -> ManualEscalationAdjustment {
    ManualEscalationAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        pinned: false,
        burst: 0,
        archived: false,
        floor_value: 0.0,
        rate: 0.0,
        channel: "".to_string(),
        name: "".to_string(),
        status: ManualEscalationAdjustmentStatus::Draft,
    }
}
