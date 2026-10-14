use super::*;

use acme_kit::text;

/// Where a measurement segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl MeasurementSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for MeasurementSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSegment {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub category: String,
    pub name: String,
    pub channel: String,
    pub expires_at: i64,
    pub label: String,
    pub status: MeasurementSegmentStatus,
}

impl MeasurementSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementSegmentStatus) -> bool {
        match self.status {
            MeasurementSegmentStatus::Draft => {
                match next {
                    MeasurementSegmentStatus::Pending | MeasurementSegmentStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementSegmentStatus::Pending => {
                match next {
                    MeasurementSegmentStatus::Suspended | MeasurementSegmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementSegmentStatus::Active => {
                match next {
                    MeasurementSegmentStatus::Suspended | MeasurementSegmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementSegmentStatus::Suspended => {
                return next == MeasurementSegmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement segment {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.expires_at as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementSegment {
        MeasurementSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            category: self.category.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            expires_at: self.expires_at,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a measurement segment in status draft.
pub fn new_measurement_segment(code: &str, title: &str) -> MeasurementSegment {
    MeasurementSegment {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        category: "".to_string(),
        name: "".to_string(),
        channel: "".to_string(),
        expires_at: 0,
        label: "".to_string(),
        status: MeasurementSegmentStatus::Draft,
    }
}
