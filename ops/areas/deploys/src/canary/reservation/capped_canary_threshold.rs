use super::*;

use acme_kit::text;

/// Where a capped canary threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedCanaryThresholdStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedCanaryThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedCanaryThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped canary threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryThreshold {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub owner_id: String,
    pub external_ref: String,
    pub rate: f64,
    pub title: String,
    pub status: CappedCanaryThresholdStatus,
}

impl CappedCanaryThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedCanaryThresholdStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedCanaryThresholdStatus) -> bool {
        match self.status {
            CappedCanaryThresholdStatus::Proposed => {
                return next == CappedCanaryThresholdStatus::Approved || next == CappedCanaryThresholdStatus::Rejected;
            }
            CappedCanaryThresholdStatus::Approved => {
                return next == CappedCanaryThresholdStatus::Withdrawn;
            }
            CappedCanaryThresholdStatus::Rejected => {
                return next == CappedCanaryThresholdStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedCanaryThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped canary threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped canary threshold", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.created_at as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedCanaryThreshold {
        CappedCanaryThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            owner_id: self.owner_id.clone(),
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped canary threshold in status proposed.
pub fn new_capped_canary_threshold(code: &str, label: &str) -> CappedCanaryThreshold {
    CappedCanaryThreshold {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        owner_id: "".to_string(),
        external_ref: "".to_string(),
        rate: 0.0,
        title: "".to_string(),
        status: CappedCanaryThresholdStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_canary_threshold("code-728", "label-313");
        assert_eq!(item.status, CappedCanaryThresholdStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_capped_canary_threshold("code-728", "label-313");
        if item.move_to(CappedCanaryThresholdStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedCanaryThresholdStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_canary_threshold("code-728", "label-313");
        item.status = CappedCanaryThresholdStatus::Withdrawn;
        assert!(
            item.move_to(CappedCanaryThresholdStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
