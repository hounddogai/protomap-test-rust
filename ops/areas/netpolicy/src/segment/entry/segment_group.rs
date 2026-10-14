use super::*;

use acme_kit::text;

/// Where a segment group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SegmentGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SegmentGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment group as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentGroup {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub principal: String,
    pub external_ref: String,
    pub created_at: i64,
    pub status: SegmentGroupStatus,
}

impl SegmentGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentGroupStatus) -> bool {
        match self.status {
            SegmentGroupStatus::Created => {
                match next {
                    SegmentGroupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentGroupStatus::Validated => {
                return next == SegmentGroupStatus::Retired;
            }
            SegmentGroupStatus::Published => {
                return next == SegmentGroupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.created_at as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentGroup {
        SegmentGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            principal: self.principal.clone(),
            external_ref: self.external_ref.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a segment group in status created.
pub fn new_segment_group(code: &str, name: &str) -> SegmentGroup {
    SegmentGroup {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        principal: "".to_string(),
        external_ref: "".to_string(),
        created_at: 0,
        status: SegmentGroupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_segment_group("code-330", "name-965");
        assert_eq!(item.status, SegmentGroupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_segment_group("code-330", "name-965");
        if item.move_to(SegmentGroupStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentGroupStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_segment_group("code-330", "name-965");
        item.status = SegmentGroupStatus::Retired;
        assert!(
            item.move_to(SegmentGroupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
