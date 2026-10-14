use super::*;

use acme_kit::text;

/// Where a manual page tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPageTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualPageTierStatus {
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

impl std::fmt::Display for ManualPageTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual page tier as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageTier {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub key_id: String,
    pub floor_value: f64,
    pub ttl_seconds: i64,
    pub tags: Vec<String>,
    pub principal: String,
    pub ceiling: f64,
    pub host: String,
    pub capacity: i64,
    pub status: ManualPageTierStatus,
}

impl ManualPageTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPageTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPageTierStatus) -> bool {
        match self.status {
            ManualPageTierStatus::New => {
                match next {
                    ManualPageTierStatus::Queued | ManualPageTierStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPageTierStatus::Queued => {
                return next == ManualPageTierStatus::Done;
            }
            ManualPageTierStatus::Running => {
                match next {
                    ManualPageTierStatus::Done | ManualPageTierStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPageTierStatus::Done => {
                return next == ManualPageTierStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualPageTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual page tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual page tier", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualPageTier {
        ManualPageTier {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            key_id: self.key_id.clone(),
            floor_value: self.floor_value,
            ttl_seconds: self.ttl_seconds,
            tags: self.tags.clone(),
            principal: self.principal.clone(),
            ceiling: self.ceiling,
            host: self.host.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual page tier in status new.
pub fn new_manual_page_tier(code: &str, name: &str) -> ManualPageTier {
    ManualPageTier {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        key_id: "".to_string(),
        floor_value: 0.0,
        ttl_seconds: 0,
        tags: Vec::new(),
        principal: "".to_string(),
        ceiling: 0.0,
        host: "".to_string(),
        capacity: 0,
        status: ManualPageTierStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_page_tier("code-227", "name-220");
        assert_eq!(item.status, ManualPageTierStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_manual_page_tier("code-227", "name-220");
        if item.move_to(ManualPageTierStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualPageTierStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_page_tier("code-227", "name-220");
        item.status = ManualPageTierStatus::Failed;
        assert!(
            item.move_to(ManualPageTierStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
