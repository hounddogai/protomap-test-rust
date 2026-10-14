use super::*;

use acme_kit::text;

/// Where a fallback peering window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPeeringWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackPeeringWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackPeeringWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback peering window as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPeeringWindow {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub unit: String,
    pub burst: i64,
    pub key_id: String,
    pub retries: i64,
    pub created_at: i64,
    pub checksum: String,
    pub score: f64,
    pub revision: i64,
    pub status: FallbackPeeringWindowStatus,
}

impl FallbackPeeringWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPeeringWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPeeringWindowStatus) -> bool {
        match self.status {
            FallbackPeeringWindowStatus::Created => {
                match next {
                    FallbackPeeringWindowStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPeeringWindowStatus::Validated => {
                return next == FallbackPeeringWindowStatus::Published || next == FallbackPeeringWindowStatus::Retired;
            }
            FallbackPeeringWindowStatus::Published => {
                match next {
                    FallbackPeeringWindowStatus::Retired => {
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

    pub fn move_to(&mut self, next: FallbackPeeringWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback peering window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback peering window", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.sequence as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPeeringWindow {
        FallbackPeeringWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            unit: self.unit.clone(),
            burst: self.burst,
            key_id: self.key_id.clone(),
            retries: self.retries,
            created_at: self.created_at,
            checksum: self.checksum.clone(),
            score: self.score,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a fallback peering window in status created.
pub fn new_fallback_peering_window(code: &str, title: &str) -> FallbackPeeringWindow {
    FallbackPeeringWindow {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        unit: "".to_string(),
        burst: 0,
        key_id: "".to_string(),
        retries: 0,
        created_at: 0,
        checksum: "".to_string(),
        score: 0.0,
        revision: 0,
        status: FallbackPeeringWindowStatus::Created,
    }
}
