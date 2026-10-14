use super::*;

use acme_kit::text;

/// Where a pending probe adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingProbeAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingProbeAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingProbeAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending probe adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAdjustment {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub capacity: i64,
    pub pinned: bool,
    pub unit: String,
    pub starts_at: i64,
    pub maximum: f64,
    pub archived: bool,
    pub locale: String,
    pub limit: i64,
    pub status: PendingProbeAdjustmentStatus,
}

impl PendingProbeAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingProbeAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingProbeAdjustmentStatus) -> bool {
        match self.status {
            PendingProbeAdjustmentStatus::Created => {
                match next {
                    PendingProbeAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingProbeAdjustmentStatus::Validated => {
                return next == PendingProbeAdjustmentStatus::Retired;
            }
            PendingProbeAdjustmentStatus::Published => {
                return next == PendingProbeAdjustmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingProbeAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending probe adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.capacity as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingProbeAdjustment {
        PendingProbeAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            capacity: self.capacity,
            pinned: self.pinned,
            unit: self.unit.clone(),
            starts_at: self.starts_at,
            maximum: self.maximum,
            archived: self.archived,
            locale: self.locale.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a pending probe adjustment in status created.
pub fn new_pending_probe_adjustment(code: &str, label: &str) -> PendingProbeAdjustment {
    PendingProbeAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        capacity: 0,
        pinned: false,
        unit: "".to_string(),
        starts_at: 0,
        maximum: 0.0,
        archived: false,
        locale: "".to_string(),
        limit: 0,
        status: PendingProbeAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_probe_adjustment("code-69", "label-241");
        assert_eq!(item.status, PendingProbeAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_pending_probe_adjustment("code-69", "label-241");
        if item.move_to(PendingProbeAdjustmentStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingProbeAdjustmentStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_probe_adjustment("code-69", "label-241");
        item.status = PendingProbeAdjustmentStatus::Retired;
        assert!(
            item.move_to(PendingProbeAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
