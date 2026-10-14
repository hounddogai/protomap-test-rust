use super::*;

use acme_kit::text;

/// Where a adaptive alert cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAlertCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveAlertCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveAlertCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive alert cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertCycle {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub country: String,
    pub minimum: f64,
    pub status: AdaptiveAlertCycleStatus,
}

impl AdaptiveAlertCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAlertCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAlertCycleStatus) -> bool {
        match self.status {
            AdaptiveAlertCycleStatus::Proposed => {
                return next == AdaptiveAlertCycleStatus::Rejected || next == AdaptiveAlertCycleStatus::Withdrawn;
            }
            AdaptiveAlertCycleStatus::Approved => {
                match next {
                    AdaptiveAlertCycleStatus::Rejected | AdaptiveAlertCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAlertCycleStatus::Rejected => {
                match next {
                    AdaptiveAlertCycleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveAlertCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive alert cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.maximum * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAlertCycle {
        AdaptiveAlertCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            country: self.country.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive alert cycle in status proposed.
pub fn new_adaptive_alert_cycle(code: &str, title: &str) -> AdaptiveAlertCycle {
    AdaptiveAlertCycle {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        country: "".to_string(),
        minimum: 0.0,
        status: AdaptiveAlertCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_alert_cycle("code-811", "title-845");
        assert_eq!(item.status, AdaptiveAlertCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_adaptive_alert_cycle("code-811", "title-845");
        if item.move_to(AdaptiveAlertCycleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveAlertCycleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_alert_cycle("code-811", "title-845");
        item.status = AdaptiveAlertCycleStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveAlertCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
