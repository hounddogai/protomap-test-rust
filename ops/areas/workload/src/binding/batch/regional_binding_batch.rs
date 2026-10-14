use super::*;

use acme_kit::text;

/// Where a regional binding batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalBindingBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalBindingBatchStatus {
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

impl std::fmt::Display for RegionalBindingBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional binding batch as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBindingBatch {
    pub code: String,
    pub title: String,
    pub region: String,
    pub source: String,
    pub weight: f64,
    pub host: String,
    pub created_at: i64,
    pub cost: f64,
    pub approved: bool,
    pub score: f64,
    pub ttl_seconds: i64,
    pub status: RegionalBindingBatchStatus,
}

impl RegionalBindingBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalBindingBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalBindingBatchStatus) -> bool {
        match self.status {
            RegionalBindingBatchStatus::New => {
                return next == RegionalBindingBatchStatus::Done || next == RegionalBindingBatchStatus::Failed;
            }
            RegionalBindingBatchStatus::Queued => {
                return next == RegionalBindingBatchStatus::Done || next == RegionalBindingBatchStatus::Failed;
            }
            RegionalBindingBatchStatus::Running => {
                return next == RegionalBindingBatchStatus::Failed;
            }
            RegionalBindingBatchStatus::Done => {
                match next {
                    RegionalBindingBatchStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionalBindingBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional binding batch {} cannot move from {} to {}",
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
        format!("{} {}", "regional binding batch", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.cost + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalBindingBatch {
        RegionalBindingBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            source: self.source.clone(),
            weight: self.weight,
            host: self.host.clone(),
            created_at: self.created_at,
            cost: self.cost,
            approved: self.approved,
            score: self.score,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional binding batch in status new.
pub fn new_regional_binding_batch(code: &str, title: &str) -> RegionalBindingBatch {
    RegionalBindingBatch {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        source: "".to_string(),
        weight: 0.0,
        host: "".to_string(),
        created_at: 0,
        cost: 0.0,
        approved: false,
        score: 0.0,
        ttl_seconds: 0,
        status: RegionalBindingBatchStatus::New,
    }
}
