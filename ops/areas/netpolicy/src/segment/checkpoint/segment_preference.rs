use super::*;

use acme_kit::text;

/// Where a segment preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SegmentPreferenceStatus {
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

impl std::fmt::Display for SegmentPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment preference as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPreference {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub starts_at: i64,
    pub weight: f64,
    pub cipher: String,
    pub reviewer: String,
    pub status: SegmentPreferenceStatus,
}

impl SegmentPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentPreferenceStatus) -> bool {
        match self.status {
            SegmentPreferenceStatus::Draft => {
                match next {
                    SegmentPreferenceStatus::Pending | SegmentPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentPreferenceStatus::Pending => {
                match next {
                    SegmentPreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentPreferenceStatus::Active => {
                return next == SegmentPreferenceStatus::Suspended;
            }
            SegmentPreferenceStatus::Suspended => {
                return next == SegmentPreferenceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment preference {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.weight * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentPreference {
        SegmentPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            starts_at: self.starts_at,
            weight: self.weight,
            cipher: self.cipher.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a segment preference in status draft.
pub fn new_segment_preference(code: &str, title: &str) -> SegmentPreference {
    SegmentPreference {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        starts_at: 0,
        weight: 0.0,
        cipher: "".to_string(),
        reviewer: "".to_string(),
        status: SegmentPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_segment_preference("code-737", "title-795");
        assert_eq!(item.status, SegmentPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_segment_preference("code-737", "title-795");
        if item.move_to(SegmentPreferenceStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentPreferenceStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_segment_preference("code-737", "title-795");
        item.status = SegmentPreferenceStatus::Closed;
        assert!(
            item.move_to(SegmentPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
