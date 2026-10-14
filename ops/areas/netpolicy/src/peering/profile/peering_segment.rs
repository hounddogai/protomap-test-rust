use super::*;

use acme_kit::text;

/// Where a peering segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PeeringSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PeeringSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering segment as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSegment {
    pub code: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub sequence: i64,
    pub unit: String,
    pub audience: String,
    pub minimum: f64,
    pub key_id: String,
    pub checksum: String,
    pub timezone: String,
    pub status: PeeringSegmentStatus,
}

impl PeeringSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringSegmentStatus) -> bool {
        match self.status {
            PeeringSegmentStatus::Created => {
                match next {
                    PeeringSegmentStatus::Validated | PeeringSegmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringSegmentStatus::Validated => {
                match next {
                    PeeringSegmentStatus::Published | PeeringSegmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringSegmentStatus::Published => {
                return next == PeeringSegmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "peering segment", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.sequence as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringSegment {
        PeeringSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            owner_id: self.owner_id.clone(),
            sequence: self.sequence,
            unit: self.unit.clone(),
            audience: self.audience.clone(),
            minimum: self.minimum,
            key_id: self.key_id.clone(),
            checksum: self.checksum.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a peering segment in status created.
pub fn new_peering_segment(code: &str, name: &str) -> PeeringSegment {
    PeeringSegment {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        owner_id: "".to_string(),
        sequence: 0,
        unit: "".to_string(),
        audience: "".to_string(),
        minimum: 0.0,
        key_id: "".to_string(),
        checksum: "".to_string(),
        timezone: "".to_string(),
        status: PeeringSegmentStatus::Created,
    }
}
