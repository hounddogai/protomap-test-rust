use super::*;

use acme_kit::text;

/// Where a tiered alert adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAlertAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredAlertAdjustmentStatus {
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

impl std::fmt::Display for TieredAlertAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered alert adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAdjustment {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub rate: f64,
    pub pinned: bool,
    pub status: TieredAlertAdjustmentStatus,
}

impl TieredAlertAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAlertAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAlertAdjustmentStatus) -> bool {
        match self.status {
            TieredAlertAdjustmentStatus::New => {
                return next == TieredAlertAdjustmentStatus::Queued || next == TieredAlertAdjustmentStatus::Done;
            }
            TieredAlertAdjustmentStatus::Queued => {
                match next {
                    TieredAlertAdjustmentStatus::Running | TieredAlertAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAlertAdjustmentStatus::Running => {
                return next == TieredAlertAdjustmentStatus::Done || next == TieredAlertAdjustmentStatus::Failed;
            }
            TieredAlertAdjustmentStatus::Done => {
                match next {
                    TieredAlertAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: TieredAlertAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered alert adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered alert adjustment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> TieredAlertAdjustment {
        TieredAlertAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            rate: self.rate,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered alert adjustment in status new.
pub fn new_tiered_alert_adjustment(code: &str, name: &str) -> TieredAlertAdjustment {
    TieredAlertAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        rate: 0.0,
        pinned: false,
        status: TieredAlertAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_alert_adjustment("code-328", "name-214");
        assert_eq!(item.status, TieredAlertAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_tiered_alert_adjustment("code-328", "name-214");
        if item.move_to(TieredAlertAdjustmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredAlertAdjustmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_alert_adjustment("code-328", "name-214");
        item.status = TieredAlertAdjustmentStatus::Failed;
        assert!(
            item.move_to(TieredAlertAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
