use super::*;

use acme_kit::text;

/// Where a lenient flow log estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFlowLogEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientFlowLogEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientFlowLogEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient flow log estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEstimate {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub threshold: f64,
    pub quantity: i64,
    pub external_ref: String,
    pub scope: String,
    pub priority: i64,
    pub sequence: i64,
    pub ttl_seconds: i64,
    pub pinned: bool,
    pub status: LenientFlowLogEstimateStatus,
}

impl LenientFlowLogEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFlowLogEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFlowLogEstimateStatus) -> bool {
        match self.status {
            LenientFlowLogEstimateStatus::Proposed => {
                return next == LenientFlowLogEstimateStatus::Approved
                    || next == LenientFlowLogEstimateStatus::Withdrawn;
            }
            LenientFlowLogEstimateStatus::Approved => {
                return next == LenientFlowLogEstimateStatus::Withdrawn;
            }
            LenientFlowLogEstimateStatus::Rejected => {
                return next == LenientFlowLogEstimateStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientFlowLogEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient flow log estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.minimum * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientFlowLogEstimate {
        LenientFlowLogEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            threshold: self.threshold,
            quantity: self.quantity,
            external_ref: self.external_ref.clone(),
            scope: self.scope.clone(),
            priority: self.priority,
            sequence: self.sequence,
            ttl_seconds: self.ttl_seconds,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient flow log estimate in status proposed.
pub fn new_lenient_flow_log_estimate(code: &str, label: &str) -> LenientFlowLogEstimate {
    LenientFlowLogEstimate {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        threshold: 0.0,
        quantity: 0,
        external_ref: "".to_string(),
        scope: "".to_string(),
        priority: 0,
        sequence: 0,
        ttl_seconds: 0,
        pinned: false,
        status: LenientFlowLogEstimateStatus::Proposed,
    }
}
