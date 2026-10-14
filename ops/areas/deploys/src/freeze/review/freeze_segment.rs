use super::*;

use acme_kit::text;

/// Where a freeze segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FreezeSegmentStatus {
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

impl std::fmt::Display for FreezeSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One freeze segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSegment {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub contact_email: String,
    pub currency: String,
    pub namespace: String,
    pub floor_value: f64,
    pub status: FreezeSegmentStatus,
}

impl FreezeSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeSegmentStatus) -> bool {
        match self.status {
            FreezeSegmentStatus::New => {
                match next {
                    FreezeSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeSegmentStatus::Queued => {
                match next {
                    FreezeSegmentStatus::Running | FreezeSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeSegmentStatus::Running => {
                match next {
                    FreezeSegmentStatus::Done | FreezeSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeSegmentStatus::Done => {
                match next {
                    FreezeSegmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: FreezeSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.floor_value + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeSegment {
        FreezeSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            contact_email: self.contact_email.clone(),
            currency: self.currency.clone(),
            namespace: self.namespace.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a freeze segment in status new.
pub fn new_freeze_segment(code: &str, label: &str) -> FreezeSegment {
    FreezeSegment {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        contact_email: "".to_string(),
        currency: "".to_string(),
        namespace: "".to_string(),
        floor_value: 0.0,
        status: FreezeSegmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_freeze_segment("code-910", "label-345");
        assert_eq!(item.status, FreezeSegmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_freeze_segment("code-910", "label-345");
        if item.move_to(FreezeSegmentStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeSegmentStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_freeze_segment("code-910", "label-345");
        item.status = FreezeSegmentStatus::Failed;
        assert!(item.move_to(FreezeSegmentStatus::New).is_err(), "expected move to fail");
    }
}
