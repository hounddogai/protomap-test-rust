use super::*;

use acme_kit::text;

/// Where a scheduled region pin override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRegionPinOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledRegionPinOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ScheduledRegionPinOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled region pin override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinOverride {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub owner_id: String,
    pub checksum: String,
    pub phone: String,
    pub quantity: i64,
    pub status: ScheduledRegionPinOverrideStatus,
}

impl ScheduledRegionPinOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRegionPinOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRegionPinOverrideStatus) -> bool {
        match self.status {
            ScheduledRegionPinOverrideStatus::Open => {
                match next {
                    ScheduledRegionPinOverrideStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRegionPinOverrideStatus::InReview => {
                match next {
                    ScheduledRegionPinOverrideStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRegionPinOverrideStatus::Resolved => {
                match next {
                    ScheduledRegionPinOverrideStatus::Reopened | ScheduledRegionPinOverrideStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRegionPinOverrideStatus::Reopened => {
                return next == ScheduledRegionPinOverrideStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledRegionPinOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled region pin override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ScheduledRegionPinOverride {
        ScheduledRegionPinOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            owner_id: self.owner_id.clone(),
            checksum: self.checksum.clone(),
            phone: self.phone.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled region pin override in status open.
pub fn new_scheduled_region_pin_override(code: &str, label: &str) -> ScheduledRegionPinOverride {
    ScheduledRegionPinOverride {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        owner_id: "".to_string(),
        checksum: "".to_string(),
        phone: "".to_string(),
        quantity: 0,
        status: ScheduledRegionPinOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_region_pin_override("code-439", "label-618");
        assert_eq!(item.status, ScheduledRegionPinOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_scheduled_region_pin_override("code-439", "label-618");
        if item.move_to(ScheduledRegionPinOverrideStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRegionPinOverrideStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_region_pin_override("code-439", "label-618");
        item.status = ScheduledRegionPinOverrideStatus::Archived;
        assert!(
            item.move_to(ScheduledRegionPinOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
