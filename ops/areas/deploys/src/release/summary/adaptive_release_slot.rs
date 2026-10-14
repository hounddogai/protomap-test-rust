use super::*;

use acme_kit::text;

/// Where a adaptive release slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveReleaseSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveReleaseSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveReleaseSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive release slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseSlot {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub discount: f64,
    pub owner_id: String,
    pub ratio: f64,
    pub rate: f64,
    pub channel: String,
    pub cluster: String,
    pub status: AdaptiveReleaseSlotStatus,
}

impl AdaptiveReleaseSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveReleaseSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveReleaseSlotStatus) -> bool {
        match self.status {
            AdaptiveReleaseSlotStatus::Proposed => {
                return next == AdaptiveReleaseSlotStatus::Withdrawn;
            }
            AdaptiveReleaseSlotStatus::Approved => {
                match next {
                    AdaptiveReleaseSlotStatus::Rejected | AdaptiveReleaseSlotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveReleaseSlotStatus::Rejected => {
                return next == AdaptiveReleaseSlotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveReleaseSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive release slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive release slot", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.discount + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveReleaseSlot {
        AdaptiveReleaseSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            discount: self.discount,
            owner_id: self.owner_id.clone(),
            ratio: self.ratio,
            rate: self.rate,
            channel: self.channel.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive release slot in status proposed.
pub fn new_adaptive_release_slot(code: &str, label: &str) -> AdaptiveReleaseSlot {
    AdaptiveReleaseSlot {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        discount: 0.0,
        owner_id: "".to_string(),
        ratio: 0.0,
        rate: 0.0,
        channel: "".to_string(),
        cluster: "".to_string(),
        status: AdaptiveReleaseSlotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_release_slot("code-166", "label-15");
        assert_eq!(item.status, AdaptiveReleaseSlotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_adaptive_release_slot("code-166", "label-15");
        if item.move_to(AdaptiveReleaseSlotStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveReleaseSlotStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_release_slot("code-166", "label-15");
        item.status = AdaptiveReleaseSlotStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveReleaseSlotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
