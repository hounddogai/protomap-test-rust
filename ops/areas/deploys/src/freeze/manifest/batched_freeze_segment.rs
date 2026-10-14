use super::*;

use acme_kit::text;

/// Where a batched freeze segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedFreezeSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedFreezeSegmentStatus {
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

impl std::fmt::Display for BatchedFreezeSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched freeze segment as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFreezeSegment {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub archived: bool,
    pub notes: String,
    pub country: String,
    pub principal: String,
    pub batch_size: i64,
    pub audience: String,
    pub tenant_id: String,
    pub status: BatchedFreezeSegmentStatus,
}

impl BatchedFreezeSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedFreezeSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedFreezeSegmentStatus) -> bool {
        match self.status {
            BatchedFreezeSegmentStatus::Draft => {
                match next {
                    BatchedFreezeSegmentStatus::Active | BatchedFreezeSegmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFreezeSegmentStatus::Pending => {
                match next {
                    BatchedFreezeSegmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFreezeSegmentStatus::Active => {
                return next == BatchedFreezeSegmentStatus::Suspended;
            }
            BatchedFreezeSegmentStatus::Suspended => {
                return next == BatchedFreezeSegmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedFreezeSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched freeze segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> BatchedFreezeSegment {
        BatchedFreezeSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            archived: self.archived,
            notes: self.notes.clone(),
            country: self.country.clone(),
            principal: self.principal.clone(),
            batch_size: self.batch_size,
            audience: self.audience.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a batched freeze segment in status draft.
pub fn new_batched_freeze_segment(code: &str, title: &str) -> BatchedFreezeSegment {
    BatchedFreezeSegment {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        archived: false,
        notes: "".to_string(),
        country: "".to_string(),
        principal: "".to_string(),
        batch_size: 0,
        audience: "".to_string(),
        tenant_id: "".to_string(),
        status: BatchedFreezeSegmentStatus::Draft,
    }
}
