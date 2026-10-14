use super::*;

use acme_kit::text;

/// Where a segment variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SegmentVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SegmentVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment variance as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentVariance {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub region: String,
    pub rate: f64,
    pub margin: f64,
    pub host: String,
    pub tags: Vec<String>,
    pub locale: String,
    pub port: i64,
    pub tenant_id: String,
    pub status: SegmentVarianceStatus,
}

impl SegmentVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentVarianceStatus) -> bool {
        match self.status {
            SegmentVarianceStatus::Proposed => {
                return next == SegmentVarianceStatus::Approved || next == SegmentVarianceStatus::Rejected;
            }
            SegmentVarianceStatus::Approved => {
                match next {
                    SegmentVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentVarianceStatus::Rejected => {
                return next == SegmentVarianceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.burst as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentVariance {
        SegmentVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            region: self.region.clone(),
            rate: self.rate,
            margin: self.margin,
            host: self.host.clone(),
            tags: self.tags.clone(),
            locale: self.locale.clone(),
            port: self.port,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a segment variance in status proposed.
pub fn new_segment_variance(code: &str, title: &str) -> SegmentVariance {
    SegmentVariance {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        region: "".to_string(),
        rate: 0.0,
        margin: 0.0,
        host: "".to_string(),
        tags: Vec::new(),
        locale: "".to_string(),
        port: 0,
        tenant_id: "".to_string(),
        status: SegmentVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_segment_variance("code-683", "title-627");
        assert_eq!(item.status, SegmentVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_segment_variance("code-683", "title-627");
        if item.move_to(SegmentVarianceStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentVarianceStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_segment_variance("code-683", "title-627");
        item.status = SegmentVarianceStatus::Withdrawn;
        assert!(
            item.move_to(SegmentVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
