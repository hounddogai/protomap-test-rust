use super::*;

use acme_kit::text;

/// Where a global alert cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalAlertCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalAlertCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalAlertCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global alert cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertCycle {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub timezone: String,
    pub ratio: f64,
    pub key_id: String,
    pub quantity: i64,
    pub currency: String,
    pub cluster: String,
    pub archived: bool,
    pub status: GlobalAlertCycleStatus,
}

impl GlobalAlertCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalAlertCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalAlertCycleStatus) -> bool {
        match self.status {
            GlobalAlertCycleStatus::Proposed => {
                return next == GlobalAlertCycleStatus::Rejected || next == GlobalAlertCycleStatus::Withdrawn;
            }
            GlobalAlertCycleStatus::Approved => {
                match next {
                    GlobalAlertCycleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAlertCycleStatus::Rejected => {
                match next {
                    GlobalAlertCycleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: GlobalAlertCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global alert cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global alert cycle", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.quantity as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalAlertCycle {
        GlobalAlertCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            timezone: self.timezone.clone(),
            ratio: self.ratio,
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            currency: self.currency.clone(),
            cluster: self.cluster.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global alert cycle in status proposed.
pub fn new_global_alert_cycle(code: &str, name: &str) -> GlobalAlertCycle {
    GlobalAlertCycle {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        timezone: "".to_string(),
        ratio: 0.0,
        key_id: "".to_string(),
        quantity: 0,
        currency: "".to_string(),
        cluster: "".to_string(),
        archived: false,
        status: GlobalAlertCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_global_alert_cycle("code-528", "name-836");
        assert_eq!(item.status, GlobalAlertCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_global_alert_cycle("code-528", "name-836");
        if item.move_to(GlobalAlertCycleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalAlertCycleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_global_alert_cycle("code-528", "name-836");
        item.status = GlobalAlertCycleStatus::Withdrawn;
        assert!(
            item.move_to(GlobalAlertCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
