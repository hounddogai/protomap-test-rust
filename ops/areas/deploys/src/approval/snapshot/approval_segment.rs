use super::*;

use acme_kit::text;

/// Where a approval segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ApprovalSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ApprovalSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval segment as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSegment {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub description: String,
    pub retries: i64,
    pub priority: i64,
    pub status: ApprovalSegmentStatus,
}

impl ApprovalSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalSegmentStatus) -> bool {
        match self.status {
            ApprovalSegmentStatus::Created => {
                match next {
                    ApprovalSegmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalSegmentStatus::Validated => {
                match next {
                    ApprovalSegmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalSegmentStatus::Published => {
                match next {
                    ApprovalSegmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: ApprovalSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "approval segment", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.retries as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalSegment {
        ApprovalSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            description: self.description.clone(),
            retries: self.retries,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a approval segment in status created.
pub fn new_approval_segment(code: &str, name: &str) -> ApprovalSegment {
    ApprovalSegment {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        description: "".to_string(),
        retries: 0,
        priority: 0,
        status: ApprovalSegmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_approval_segment("code-275", "name-271");
        assert_eq!(item.status, ApprovalSegmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_approval_segment("code-275", "name-271");
        if item.move_to(ApprovalSegmentStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalSegmentStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_approval_segment("code-275", "name-271");
        item.status = ApprovalSegmentStatus::Retired;
        assert!(
            item.move_to(ApprovalSegmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
