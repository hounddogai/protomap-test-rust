use super::*;

use acme_kit::text;

/// Where a primary timeline variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTimelineVarianceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryTimelineVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryTimelineVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary timeline variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTimelineVariance {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub unit: String,
    pub archived: bool,
    pub channel: String,
    pub owner_id: String,
    pub status: PrimaryTimelineVarianceStatus,
}

impl PrimaryTimelineVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTimelineVarianceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTimelineVarianceStatus) -> bool {
        match self.status {
            PrimaryTimelineVarianceStatus::Created => {
                return next == PrimaryTimelineVarianceStatus::Published
                    || next == PrimaryTimelineVarianceStatus::Retired;
            }
            PrimaryTimelineVarianceStatus::Validated => {
                return next == PrimaryTimelineVarianceStatus::Published;
            }
            PrimaryTimelineVarianceStatus::Published => {
                match next {
                    PrimaryTimelineVarianceStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryTimelineVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary timeline variance {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> PrimaryTimelineVariance {
        PrimaryTimelineVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            unit: self.unit.clone(),
            archived: self.archived,
            channel: self.channel.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary timeline variance in status created.
pub fn new_primary_timeline_variance(code: &str, title: &str) -> PrimaryTimelineVariance {
    PrimaryTimelineVariance {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        unit: "".to_string(),
        archived: false,
        channel: "".to_string(),
        owner_id: "".to_string(),
        status: PrimaryTimelineVarianceStatus::Created,
    }
}
