use super::*;

use acme_kit::text;

/// Where a suspension segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SuspensionSegmentStatus {
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

impl std::fmt::Display for SuspensionSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension segment as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSegment {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub reviewer: String,
    pub revision: i64,
    pub archived: bool,
    pub status: SuspensionSegmentStatus,
}

impl SuspensionSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionSegmentStatus) -> bool {
        match self.status {
            SuspensionSegmentStatus::New => {
                match next {
                    SuspensionSegmentStatus::Running | SuspensionSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionSegmentStatus::Queued => {
                match next {
                    SuspensionSegmentStatus::Running | SuspensionSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionSegmentStatus::Running => {
                return next == SuspensionSegmentStatus::Done || next == SuspensionSegmentStatus::Failed;
            }
            SuspensionSegmentStatus::Done => {
                return next == SuspensionSegmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "suspension segment", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.discount - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionSegment {
        SuspensionSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            reviewer: self.reviewer.clone(),
            revision: self.revision,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a suspension segment in status new.
pub fn new_suspension_segment(code: &str, name: &str) -> SuspensionSegment {
    SuspensionSegment {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        reviewer: "".to_string(),
        revision: 0,
        archived: false,
        status: SuspensionSegmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_suspension_segment("code-746", "name-993");
        assert_eq!(item.status, SuspensionSegmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_suspension_segment("code-746", "name-993");
        if item.move_to(SuspensionSegmentStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionSegmentStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_suspension_segment("code-746", "name-993");
        item.status = SuspensionSegmentStatus::Failed;
        assert!(
            item.move_to(SuspensionSegmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
