use super::*;

use acme_kit::text;

/// Where a segment mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SegmentMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for SegmentMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment mapping as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMapping {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub label: String,
    pub priority: i64,
    pub approved: bool,
    pub margin: f64,
    pub cluster: String,
    pub sequence: i64,
    pub reviewer: String,
    pub status: SegmentMappingStatus,
}

impl SegmentMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentMappingStatus) -> bool {
        match self.status {
            SegmentMappingStatus::Idle => {
                match next {
                    SegmentMappingStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentMappingStatus::Warming => {
                match next {
                    SegmentMappingStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentMappingStatus::Ready => {
                match next {
                    SegmentMappingStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentMappingStatus::Draining => {
                match next {
                    SegmentMappingStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SegmentMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.sequence as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentMapping {
        SegmentMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            label: self.label.clone(),
            priority: self.priority,
            approved: self.approved,
            margin: self.margin,
            cluster: self.cluster.clone(),
            sequence: self.sequence,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a segment mapping in status idle.
pub fn new_segment_mapping(code: &str, title: &str) -> SegmentMapping {
    SegmentMapping {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        label: "".to_string(),
        priority: 0,
        approved: false,
        margin: 0.0,
        cluster: "".to_string(),
        sequence: 0,
        reviewer: "".to_string(),
        status: SegmentMappingStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_segment_mapping("code-93", "title-156");
        assert_eq!(item.status, SegmentMappingStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_segment_mapping("code-93", "title-156");
        if item.move_to(SegmentMappingStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentMappingStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_segment_mapping("code-93", "title-156");
        item.status = SegmentMappingStatus::Stopped;
        assert!(
            item.move_to(SegmentMappingStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
