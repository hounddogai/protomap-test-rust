use super::*;

use acme_kit::text;

/// Where a regional timeline cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalTimelineCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalTimelineCycleStatus {
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

impl std::fmt::Display for RegionalTimelineCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional timeline cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineCycle {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub region: String,
    pub revision: i64,
    pub amount: f64,
    pub checksum: String,
    pub phone: String,
    pub status: RegionalTimelineCycleStatus,
}

impl RegionalTimelineCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalTimelineCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalTimelineCycleStatus) -> bool {
        match self.status {
            RegionalTimelineCycleStatus::Draft => {
                match next {
                    RegionalTimelineCycleStatus::Pending | RegionalTimelineCycleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalTimelineCycleStatus::Pending => {
                return next == RegionalTimelineCycleStatus::Suspended || next == RegionalTimelineCycleStatus::Closed;
            }
            RegionalTimelineCycleStatus::Active => {
                return next == RegionalTimelineCycleStatus::Closed;
            }
            RegionalTimelineCycleStatus::Suspended => {
                return next == RegionalTimelineCycleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalTimelineCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional timeline cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional timeline cycle", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.amount * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalTimelineCycle {
        RegionalTimelineCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            region: self.region.clone(),
            revision: self.revision,
            amount: self.amount,
            checksum: self.checksum.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional timeline cycle in status draft.
pub fn new_regional_timeline_cycle(code: &str, title: &str) -> RegionalTimelineCycle {
    RegionalTimelineCycle {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        region: "".to_string(),
        revision: 0,
        amount: 0.0,
        checksum: "".to_string(),
        phone: "".to_string(),
        status: RegionalTimelineCycleStatus::Draft,
    }
}
