use super::*;

use acme_kit::text;

/// Where a primary segment group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySegmentGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimarySegmentGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimarySegmentGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary segment group as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySegmentGroup {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub visible: bool,
    pub expires_at: i64,
    pub quantity: i64,
    pub currency: String,
    pub owner_id: String,
    pub status: PrimarySegmentGroupStatus,
}

impl PrimarySegmentGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySegmentGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySegmentGroupStatus) -> bool {
        match self.status {
            PrimarySegmentGroupStatus::Created => {
                match next {
                    PrimarySegmentGroupStatus::Validated | PrimarySegmentGroupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySegmentGroupStatus::Validated => {
                match next {
                    PrimarySegmentGroupStatus::Published | PrimarySegmentGroupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySegmentGroupStatus::Published => {
                match next {
                    PrimarySegmentGroupStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimarySegmentGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary segment group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.quantity as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySegmentGroup {
        PrimarySegmentGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            visible: self.visible,
            expires_at: self.expires_at,
            quantity: self.quantity,
            currency: self.currency.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary segment group in status created.
pub fn new_primary_segment_group(code: &str, label: &str) -> PrimarySegmentGroup {
    PrimarySegmentGroup {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        visible: false,
        expires_at: 0,
        quantity: 0,
        currency: "".to_string(),
        owner_id: "".to_string(),
        status: PrimarySegmentGroupStatus::Created,
    }
}
