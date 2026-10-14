use super::*;

use acme_kit::text;

/// Where a sink segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SinkSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SinkSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkSegment {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub updated_at: i64,
    pub locale: String,
    pub reviewer: String,
    pub status: SinkSegmentStatus,
}

impl SinkSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkSegmentStatus) -> bool {
        match self.status {
            SinkSegmentStatus::Proposed => {
                match next {
                    SinkSegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkSegmentStatus::Approved => {
                match next {
                    SinkSegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkSegmentStatus::Rejected => {
                return next == SinkSegmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> SinkSegment {
        SinkSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            updated_at: self.updated_at,
            locale: self.locale.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a sink segment in status proposed.
pub fn new_sink_segment(code: &str, label: &str) -> SinkSegment {
    SinkSegment {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        updated_at: 0,
        locale: "".to_string(),
        reviewer: "".to_string(),
        status: SinkSegmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_sink_segment("code-273", "label-680");
        assert_eq!(item.status, SinkSegmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_sink_segment("code-273", "label-680");
        if item.move_to(SinkSegmentStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkSegmentStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_sink_segment("code-273", "label-680");
        item.status = SinkSegmentStatus::Withdrawn;
        assert!(
            item.move_to(SinkSegmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
