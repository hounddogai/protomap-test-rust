use super::*;

use acme_kit::text;

/// Where a adaptive region pin digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRegionPinDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveRegionPinDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveRegionPinDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive region pin digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinDigest {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub ceiling: f64,
    pub reviewer: String,
    pub created_at: i64,
    pub status: AdaptiveRegionPinDigestStatus,
}

impl AdaptiveRegionPinDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRegionPinDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRegionPinDigestStatus) -> bool {
        match self.status {
            AdaptiveRegionPinDigestStatus::Proposed => {
                return next == AdaptiveRegionPinDigestStatus::Rejected;
            }
            AdaptiveRegionPinDigestStatus::Approved => {
                match next {
                    AdaptiveRegionPinDigestStatus::Rejected | AdaptiveRegionPinDigestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinDigestStatus::Rejected => {
                return next == AdaptiveRegionPinDigestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRegionPinDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive region pin digest {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ceiling - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRegionPinDigest {
        AdaptiveRegionPinDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            ceiling: self.ceiling,
            reviewer: self.reviewer.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive region pin digest in status proposed.
pub fn new_adaptive_region_pin_digest(code: &str, label: &str) -> AdaptiveRegionPinDigest {
    AdaptiveRegionPinDigest {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        ceiling: 0.0,
        reviewer: "".to_string(),
        created_at: 0,
        status: AdaptiveRegionPinDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_region_pin_digest("code-226", "label-763");
        assert_eq!(item.status, AdaptiveRegionPinDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_adaptive_region_pin_digest("code-226", "label-763");
        if item.move_to(AdaptiveRegionPinDigestStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRegionPinDigestStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_region_pin_digest("code-226", "label-763");
        item.status = AdaptiveRegionPinDigestStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveRegionPinDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
