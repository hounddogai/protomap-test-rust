use super::*;

use acme_kit::text;

/// Where a global binding segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalBindingSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalBindingSegmentStatus {
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

impl std::fmt::Display for GlobalBindingSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global binding segment as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingSegment {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub burst: i64,
    pub scope: String,
    pub status: GlobalBindingSegmentStatus,
}

impl GlobalBindingSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalBindingSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalBindingSegmentStatus) -> bool {
        match self.status {
            GlobalBindingSegmentStatus::New => {
                match next {
                    GlobalBindingSegmentStatus::Queued | GlobalBindingSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalBindingSegmentStatus::Queued => {
                return next == GlobalBindingSegmentStatus::Done;
            }
            GlobalBindingSegmentStatus::Running => {
                return next == GlobalBindingSegmentStatus::Failed;
            }
            GlobalBindingSegmentStatus::Done => {
                return next == GlobalBindingSegmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalBindingSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global binding segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global binding segment", self.code)
    }

    pub fn net_value(&self) -> f64 {
        (self.rate * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalBindingSegment {
        GlobalBindingSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            burst: self.burst,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a global binding segment in status new.
pub fn new_global_binding_segment(code: &str, title: &str) -> GlobalBindingSegment {
    GlobalBindingSegment {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        burst: 0,
        scope: "".to_string(),
        status: GlobalBindingSegmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_binding_segment("code-634", "title-758");
        assert_eq!(item.status, GlobalBindingSegmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_global_binding_segment("code-634", "title-758");
        if item.move_to(GlobalBindingSegmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalBindingSegmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_binding_segment("code-634", "title-758");
        item.status = GlobalBindingSegmentStatus::Failed;
        assert!(
            item.move_to(GlobalBindingSegmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
