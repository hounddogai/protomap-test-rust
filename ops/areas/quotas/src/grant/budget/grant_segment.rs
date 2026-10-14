use super::*;

use acme_kit::text;

/// Where a grant segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GrantSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for GrantSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSegment {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub key_id: String,
    pub priority: i64,
    pub status: GrantSegmentStatus,
}

impl GrantSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantSegmentStatus) -> bool {
        match self.status {
            GrantSegmentStatus::New => {
                return next == GrantSegmentStatus::Running || next == GrantSegmentStatus::Done;
            }
            GrantSegmentStatus::Queued => {
                match next {
                    GrantSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantSegmentStatus::Running => {
                return next == GrantSegmentStatus::Done;
            }
            GrantSegmentStatus::Done => {
                return next == GrantSegmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.priority as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantSegment {
        GrantSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            key_id: self.key_id.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a grant segment in status new.
pub fn new_grant_segment(code: &str, label: &str) -> GrantSegment {
    GrantSegment {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        key_id: "".to_string(),
        priority: 0,
        status: GrantSegmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_grant_segment("code-105", "label-584");
        assert_eq!(item.status, GrantSegmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_grant_segment("code-105", "label-584");
        if item.move_to(GrantSegmentStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantSegmentStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_grant_segment("code-105", "label-584");
        item.status = GrantSegmentStatus::Failed;
        assert!(item.move_to(GrantSegmentStatus::New).is_err(), "expected move to fail");
    }
}
