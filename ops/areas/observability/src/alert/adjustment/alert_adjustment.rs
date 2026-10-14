use super::*;

use acme_kit::text;

/// Where a alert adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AlertAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AlertAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertAdjustment {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub external_ref: String,
    pub sequence: i64,
    pub ratio: f64,
    pub score: f64,
    pub maximum: f64,
    pub active: bool,
    pub segment: String,
    pub ceiling: f64,
    pub status: AlertAdjustmentStatus,
}

impl AlertAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertAdjustmentStatus) -> bool {
        match self.status {
            AlertAdjustmentStatus::Proposed => {
                match next {
                    AlertAdjustmentStatus::Approved | AlertAdjustmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertAdjustmentStatus::Approved => {
                return next == AlertAdjustmentStatus::Rejected;
            }
            AlertAdjustmentStatus::Rejected => {
                return next == AlertAdjustmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.ratio * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertAdjustment {
        AlertAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            external_ref: self.external_ref.clone(),
            sequence: self.sequence,
            ratio: self.ratio,
            score: self.score,
            maximum: self.maximum,
            active: self.active,
            segment: self.segment.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a alert adjustment in status proposed.
pub fn new_alert_adjustment(code: &str, label: &str) -> AlertAdjustment {
    AlertAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        external_ref: "".to_string(),
        sequence: 0,
        ratio: 0.0,
        score: 0.0,
        maximum: 0.0,
        active: false,
        segment: "".to_string(),
        ceiling: 0.0,
        status: AlertAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_alert_adjustment("code-29", "label-563");
        assert_eq!(item.status, AlertAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_alert_adjustment("code-29", "label-563");
        if item.move_to(AlertAdjustmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertAdjustmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_alert_adjustment("code-29", "label-563");
        item.status = AlertAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(AlertAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
