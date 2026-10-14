use super::*;

use acme_kit::text;

/// Where a regional usage segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalUsageSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalUsageSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RegionalUsageSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional usage segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSegment {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub audience: String,
    pub host: String,
    pub cipher: String,
    pub quantity: i64,
    pub status: RegionalUsageSegmentStatus,
}

impl RegionalUsageSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalUsageSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalUsageSegmentStatus) -> bool {
        match self.status {
            RegionalUsageSegmentStatus::Open => {
                match next {
                    RegionalUsageSegmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalUsageSegmentStatus::InReview => {
                match next {
                    RegionalUsageSegmentStatus::Resolved | RegionalUsageSegmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalUsageSegmentStatus::Resolved => {
                return next == RegionalUsageSegmentStatus::Reopened;
            }
            RegionalUsageSegmentStatus::Reopened => {
                match next {
                    RegionalUsageSegmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalUsageSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional usage segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> RegionalUsageSegment {
        RegionalUsageSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            audience: self.audience.clone(),
            host: self.host.clone(),
            cipher: self.cipher.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional usage segment in status open.
pub fn new_regional_usage_segment(code: &str, name: &str) -> RegionalUsageSegment {
    RegionalUsageSegment {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        audience: "".to_string(),
        host: "".to_string(),
        cipher: "".to_string(),
        quantity: 0,
        status: RegionalUsageSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_usage_segment("code-100", "name-928");
        assert_eq!(item.status, RegionalUsageSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_regional_usage_segment("code-100", "name-928");
        if item.move_to(RegionalUsageSegmentStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalUsageSegmentStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_usage_segment("code-100", "name-928");
        item.status = RegionalUsageSegmentStatus::Archived;
        assert!(
            item.move_to(RegionalUsageSegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
