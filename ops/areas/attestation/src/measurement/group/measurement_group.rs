use super::*;

use acme_kit::text;

/// Where a measurement group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for MeasurementGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One measurement group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementGroup {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub cluster: String,
    pub weight: f64,
    pub created_at: i64,
    pub limit: i64,
    pub score: f64,
    pub checksum: String,
    pub status: MeasurementGroupStatus,
}

impl MeasurementGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementGroupStatus) -> bool {
        match self.status {
            MeasurementGroupStatus::Idle => {
                match next {
                    MeasurementGroupStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementGroupStatus::Warming => {
                return next == MeasurementGroupStatus::Ready || next == MeasurementGroupStatus::Draining;
            }
            MeasurementGroupStatus::Ready => {
                match next {
                    MeasurementGroupStatus::Draining | MeasurementGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementGroupStatus::Draining => {
                match next {
                    MeasurementGroupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: MeasurementGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement group {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.limit as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementGroup {
        MeasurementGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            cluster: self.cluster.clone(),
            weight: self.weight,
            created_at: self.created_at,
            limit: self.limit,
            score: self.score,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a measurement group in status idle.
pub fn new_measurement_group(code: &str, name: &str) -> MeasurementGroup {
    MeasurementGroup {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        cluster: "".to_string(),
        weight: 0.0,
        created_at: 0,
        limit: 0,
        score: 0.0,
        checksum: "".to_string(),
        status: MeasurementGroupStatus::Idle,
    }
}
