use super::*;

use acme_kit::text;

/// Where a pending canary mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingCanaryMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingCanaryMappingStatus {
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

impl std::fmt::Display for PendingCanaryMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending canary mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryMapping {
    pub code: String,
    pub title: String,
    pub country: String,
    pub ratio: f64,
    pub starts_at: i64,
    pub capacity: i64,
    pub issuer: String,
    pub tags: Vec<String>,
    pub status: PendingCanaryMappingStatus,
}

impl PendingCanaryMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingCanaryMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingCanaryMappingStatus) -> bool {
        match self.status {
            PendingCanaryMappingStatus::New => {
                match next {
                    PendingCanaryMappingStatus::Queued | PendingCanaryMappingStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingCanaryMappingStatus::Queued => {
                return next == PendingCanaryMappingStatus::Done;
            }
            PendingCanaryMappingStatus::Running => {
                match next {
                    PendingCanaryMappingStatus::Done | PendingCanaryMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingCanaryMappingStatus::Done => {
                return next == PendingCanaryMappingStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingCanaryMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending canary mapping {} cannot move from {} to {}",
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
        format!("{} {}", "pending canary mapping", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.starts_at as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingCanaryMapping {
        PendingCanaryMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            ratio: self.ratio,
            starts_at: self.starts_at,
            capacity: self.capacity,
            issuer: self.issuer.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a pending canary mapping in status new.
pub fn new_pending_canary_mapping(code: &str, title: &str) -> PendingCanaryMapping {
    PendingCanaryMapping {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        ratio: 0.0,
        starts_at: 0,
        capacity: 0,
        issuer: "".to_string(),
        tags: Vec::new(),
        status: PendingCanaryMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_canary_mapping("code-489", "title-190");
        assert_eq!(item.status, PendingCanaryMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_pending_canary_mapping("code-489", "title-190");
        if item.move_to(PendingCanaryMappingStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingCanaryMappingStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_canary_mapping("code-489", "title-190");
        item.status = PendingCanaryMappingStatus::Failed;
        assert!(
            item.move_to(PendingCanaryMappingStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
