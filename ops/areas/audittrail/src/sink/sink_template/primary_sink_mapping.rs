use super::*;

use acme_kit::text;

/// Where a primary sink mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySinkMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimarySinkMappingStatus {
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

impl std::fmt::Display for PrimarySinkMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary sink mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkMapping {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub quota_units: i64,
    pub active: bool,
    pub score: f64,
    pub key_id: String,
    pub status: PrimarySinkMappingStatus,
}

impl PrimarySinkMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySinkMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySinkMappingStatus) -> bool {
        match self.status {
            PrimarySinkMappingStatus::New => {
                return next == PrimarySinkMappingStatus::Running || next == PrimarySinkMappingStatus::Failed;
            }
            PrimarySinkMappingStatus::Queued => {
                match next {
                    PrimarySinkMappingStatus::Running | PrimarySinkMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySinkMappingStatus::Running => {
                return next == PrimarySinkMappingStatus::Failed;
            }
            PrimarySinkMappingStatus::Done => {
                match next {
                    PrimarySinkMappingStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimarySinkMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary sink mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.maximum + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySinkMapping {
        PrimarySinkMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            quota_units: self.quota_units,
            active: self.active,
            score: self.score,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary sink mapping in status new.
pub fn new_primary_sink_mapping(code: &str, title: &str) -> PrimarySinkMapping {
    PrimarySinkMapping {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        quota_units: 0,
        active: false,
        score: 0.0,
        key_id: "".to_string(),
        status: PrimarySinkMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_sink_mapping("code-69", "title-834");
        assert_eq!(item.status, PrimarySinkMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_primary_sink_mapping("code-69", "title-834");
        if item.move_to(PrimarySinkMappingStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySinkMappingStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_sink_mapping("code-69", "title-834");
        item.status = PrimarySinkMappingStatus::Failed;
        assert!(
            item.move_to(PrimarySinkMappingStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
