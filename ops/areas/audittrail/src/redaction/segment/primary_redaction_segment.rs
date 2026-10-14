use super::*;

use acme_kit::text;

/// Where a primary redaction segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRedactionSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryRedactionSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryRedactionSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary redaction segment as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionSegment {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub retries: i64,
    pub namespace: String,
    pub unit: String,
    pub expires_at: i64,
    pub weight: f64,
    pub status: PrimaryRedactionSegmentStatus,
}

impl PrimaryRedactionSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRedactionSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRedactionSegmentStatus) -> bool {
        match self.status {
            PrimaryRedactionSegmentStatus::Created => {
                return next == PrimaryRedactionSegmentStatus::Published
                    || next == PrimaryRedactionSegmentStatus::Retired;
            }
            PrimaryRedactionSegmentStatus::Validated => {
                match next {
                    PrimaryRedactionSegmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRedactionSegmentStatus::Published => {
                match next {
                    PrimaryRedactionSegmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryRedactionSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary redaction segment {} cannot move from {} to {}",
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

    pub fn effective_value(&self) -> f64 {
        ((self.expires_at as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRedactionSegment {
        PrimaryRedactionSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            retries: self.retries,
            namespace: self.namespace.clone(),
            unit: self.unit.clone(),
            expires_at: self.expires_at,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a primary redaction segment in status created.
pub fn new_primary_redaction_segment(code: &str, title: &str) -> PrimaryRedactionSegment {
    PrimaryRedactionSegment {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        retries: 0,
        namespace: "".to_string(),
        unit: "".to_string(),
        expires_at: 0,
        weight: 0.0,
        status: PrimaryRedactionSegmentStatus::Created,
    }
}
