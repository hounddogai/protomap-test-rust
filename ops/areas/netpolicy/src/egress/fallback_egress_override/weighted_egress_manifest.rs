use super::*;

use acme_kit::text;

/// Where a weighted egress manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedEgressManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedEgressManifestStatus {
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

impl std::fmt::Display for WeightedEgressManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted egress manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEgressManifest {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub retries: i64,
    pub principal: String,
    pub unit: String,
    pub version: i64,
    pub sequence: i64,
    pub cost: f64,
    pub source: String,
    pub status: WeightedEgressManifestStatus,
}

impl WeightedEgressManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedEgressManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedEgressManifestStatus) -> bool {
        match self.status {
            WeightedEgressManifestStatus::Draft => {
                return next == WeightedEgressManifestStatus::Pending || next == WeightedEgressManifestStatus::Active;
            }
            WeightedEgressManifestStatus::Pending => {
                match next {
                    WeightedEgressManifestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedEgressManifestStatus::Active => {
                match next {
                    WeightedEgressManifestStatus::Suspended | WeightedEgressManifestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedEgressManifestStatus::Suspended => {
                match next {
                    WeightedEgressManifestStatus::Closed => {
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

    pub fn move_to(&mut self, next: WeightedEgressManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted egress manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted egress manifest", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.retries as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedEgressManifest {
        WeightedEgressManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            retries: self.retries,
            principal: self.principal.clone(),
            unit: self.unit.clone(),
            version: self.version,
            sequence: self.sequence,
            cost: self.cost,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a weighted egress manifest in status draft.
pub fn new_weighted_egress_manifest(code: &str, title: &str) -> WeightedEgressManifest {
    WeightedEgressManifest {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        retries: 0,
        principal: "".to_string(),
        unit: "".to_string(),
        version: 0,
        sequence: 0,
        cost: 0.0,
        source: "".to_string(),
        status: WeightedEgressManifestStatus::Draft,
    }
}
