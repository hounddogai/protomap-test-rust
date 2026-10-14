use super::*;

use acme_kit::text;

/// Where a segment request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentRequestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SegmentRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SegmentRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRequest {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub principal: String,
    pub margin: f64,
    pub batch_size: i64,
    pub starts_at: i64,
    pub burst: i64,
    pub region: String,
    pub status: SegmentRequestStatus,
}

impl SegmentRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentRequestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentRequestStatus) -> bool {
        match self.status {
            SegmentRequestStatus::Planned => {
                match next {
                    SegmentRequestStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentRequestStatus::Scheduled => {
                match next {
                    SegmentRequestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentRequestStatus::InProgress => {
                match next {
                    SegmentRequestStatus::Completed | SegmentRequestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentRequestStatus::Completed => {
                match next {
                    SegmentRequestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SegmentRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "segment request", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.margin - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentRequest {
        SegmentRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            principal: self.principal.clone(),
            margin: self.margin,
            batch_size: self.batch_size,
            starts_at: self.starts_at,
            burst: self.burst,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a segment request in status planned.
pub fn new_segment_request(code: &str, label: &str) -> SegmentRequest {
    SegmentRequest {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        principal: "".to_string(),
        margin: 0.0,
        batch_size: 0,
        starts_at: 0,
        burst: 0,
        region: "".to_string(),
        status: SegmentRequestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_segment_request("code-573", "label-528");
        assert_eq!(item.status, SegmentRequestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_segment_request("code-573", "label-528");
        if item.move_to(SegmentRequestStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentRequestStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_segment_request("code-573", "label-528");
        item.status = SegmentRequestStatus::Cancelled;
        assert!(
            item.move_to(SegmentRequestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
