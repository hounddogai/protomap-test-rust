use super::*;

use acme_kit::text;

/// Where a segment batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SegmentBatchStatus {
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

impl std::fmt::Display for SegmentBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBatch {
    pub code: String,
    pub name: String,
    pub category: String,
    pub issuer: String,
    pub maximum: f64,
    pub port: i64,
    pub tags: Vec<String>,
    pub status: SegmentBatchStatus,
}

impl SegmentBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentBatchStatus) -> bool {
        match self.status {
            SegmentBatchStatus::Draft => {
                match next {
                    SegmentBatchStatus::Pending | SegmentBatchStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentBatchStatus::Pending => {
                return next == SegmentBatchStatus::Closed;
            }
            SegmentBatchStatus::Active => {
                match next {
                    SegmentBatchStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentBatchStatus::Suspended => {
                match next {
                    SegmentBatchStatus::Closed => {
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

    pub fn move_to(&mut self, next: SegmentBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_total(&self) -> f64 {
        ((self.maximum + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentBatch {
        SegmentBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            issuer: self.issuer.clone(),
            maximum: self.maximum,
            port: self.port,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a segment batch in status draft.
pub fn new_segment_batch(code: &str, name: &str) -> SegmentBatch {
    SegmentBatch {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        issuer: "".to_string(),
        maximum: 0.0,
        port: 0,
        tags: Vec::new(),
        status: SegmentBatchStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_segment_batch("code-297", "name-350");
        assert_eq!(item.status, SegmentBatchStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_segment_batch("code-297", "name-350");
        if item.move_to(SegmentBatchStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentBatchStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_segment_batch("code-297", "name-350");
        item.status = SegmentBatchStatus::Closed;
        assert!(
            item.move_to(SegmentBatchStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
