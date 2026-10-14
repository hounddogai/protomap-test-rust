use super::*;

use acme_kit::text;

/// Where a weighted federation cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFederationCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedFederationCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for WeightedFederationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted federation cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationCycle {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub revision: i64,
    pub unit: String,
    pub source: String,
    pub notes: String,
    pub starts_at: i64,
    pub archived: bool,
    pub score: f64,
    pub threshold: f64,
    pub status: WeightedFederationCycleStatus,
}

impl WeightedFederationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFederationCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFederationCycleStatus) -> bool {
        match self.status {
            WeightedFederationCycleStatus::New => {
                match next {
                    WeightedFederationCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFederationCycleStatus::Queued => {
                return next == WeightedFederationCycleStatus::Failed;
            }
            WeightedFederationCycleStatus::Running => {
                match next {
                    WeightedFederationCycleStatus::Done | WeightedFederationCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFederationCycleStatus::Done => {
                match next {
                    WeightedFederationCycleStatus::Failed => {
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

    pub fn move_to(&mut self, next: WeightedFederationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted federation cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted federation cycle", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.threshold + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFederationCycle {
        WeightedFederationCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            revision: self.revision,
            unit: self.unit.clone(),
            source: self.source.clone(),
            notes: self.notes.clone(),
            starts_at: self.starts_at,
            archived: self.archived,
            score: self.score,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a weighted federation cycle in status new.
pub fn new_weighted_federation_cycle(code: &str, title: &str) -> WeightedFederationCycle {
    WeightedFederationCycle {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        revision: 0,
        unit: "".to_string(),
        source: "".to_string(),
        notes: "".to_string(),
        starts_at: 0,
        archived: false,
        score: 0.0,
        threshold: 0.0,
        status: WeightedFederationCycleStatus::New,
    }
}
