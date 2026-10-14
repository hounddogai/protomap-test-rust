use super::*;

use acme_kit::text;

/// Where a adaptive limit entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveLimitEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveLimitEntryStatus {
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

impl std::fmt::Display for AdaptiveLimitEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive limit entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitEntry {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub rotation_days: i64,
    pub port: i64,
    pub locale: String,
    pub priority: i64,
    pub region: String,
    pub status: AdaptiveLimitEntryStatus,
}

impl AdaptiveLimitEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveLimitEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveLimitEntryStatus) -> bool {
        match self.status {
            AdaptiveLimitEntryStatus::New => {
                match next {
                    AdaptiveLimitEntryStatus::Running | AdaptiveLimitEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveLimitEntryStatus::Queued => {
                return next == AdaptiveLimitEntryStatus::Running;
            }
            AdaptiveLimitEntryStatus::Running => {
                return next == AdaptiveLimitEntryStatus::Done || next == AdaptiveLimitEntryStatus::Failed;
            }
            AdaptiveLimitEntryStatus::Done => {
                match next {
                    AdaptiveLimitEntryStatus::Failed => {
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

    pub fn move_to(&mut self, next: AdaptiveLimitEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive limit entry {} cannot move from {} to {}",
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
        format!("{} {}", "adaptive limit entry", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.rotation_days as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveLimitEntry {
        AdaptiveLimitEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            rotation_days: self.rotation_days,
            port: self.port,
            locale: self.locale.clone(),
            priority: self.priority,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a adaptive limit entry in status new.
pub fn new_adaptive_limit_entry(code: &str, name: &str) -> AdaptiveLimitEntry {
    AdaptiveLimitEntry {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        rotation_days: 0,
        port: 0,
        locale: "".to_string(),
        priority: 0,
        region: "".to_string(),
        status: AdaptiveLimitEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_limit_entry("code-609", "name-72");
        assert_eq!(item.status, AdaptiveLimitEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_adaptive_limit_entry("code-609", "name-72");
        if item.move_to(AdaptiveLimitEntryStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveLimitEntryStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_limit_entry("code-609", "name-72");
        item.status = AdaptiveLimitEntryStatus::Failed;
        assert!(
            item.move_to(AdaptiveLimitEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
