use super::*;

use acme_kit::text;

/// Where a fallback peering adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPeeringAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackPeeringAdjustmentStatus {
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

impl std::fmt::Display for FallbackPeeringAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback peering adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPeeringAdjustment {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub country: String,
    pub ceiling: f64,
    pub cluster: String,
    pub version: i64,
    pub external_ref: String,
    pub discount: f64,
    pub threshold: f64,
    pub status: FallbackPeeringAdjustmentStatus,
}

impl FallbackPeeringAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPeeringAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPeeringAdjustmentStatus) -> bool {
        match self.status {
            FallbackPeeringAdjustmentStatus::Planned => {
                match next {
                    FallbackPeeringAdjustmentStatus::InProgress | FallbackPeeringAdjustmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPeeringAdjustmentStatus::Scheduled => {
                match next {
                    FallbackPeeringAdjustmentStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPeeringAdjustmentStatus::InProgress => {
                return next == FallbackPeeringAdjustmentStatus::Completed
                    || next == FallbackPeeringAdjustmentStatus::Cancelled;
            }
            FallbackPeeringAdjustmentStatus::Completed => {
                return next == FallbackPeeringAdjustmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPeeringAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback peering adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback peering adjustment", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ceiling - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPeeringAdjustment {
        FallbackPeeringAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            country: self.country.clone(),
            ceiling: self.ceiling,
            cluster: self.cluster.clone(),
            version: self.version,
            external_ref: self.external_ref.clone(),
            discount: self.discount,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback peering adjustment in status planned.
pub fn new_fallback_peering_adjustment(code: &str, title: &str) -> FallbackPeeringAdjustment {
    FallbackPeeringAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        country: "".to_string(),
        ceiling: 0.0,
        cluster: "".to_string(),
        version: 0,
        external_ref: "".to_string(),
        discount: 0.0,
        threshold: 0.0,
        status: FallbackPeeringAdjustmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_peering_adjustment("code-83", "title-29");
        assert_eq!(item.status, FallbackPeeringAdjustmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_fallback_peering_adjustment("code-83", "title-29");
        if item.move_to(FallbackPeeringAdjustmentStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPeeringAdjustmentStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_peering_adjustment("code-83", "title-29");
        item.status = FallbackPeeringAdjustmentStatus::Cancelled;
        assert!(
            item.move_to(FallbackPeeringAdjustmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
