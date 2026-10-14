use super::*;

use acme_kit::text;

/// Where a limit entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LimitEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LimitEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitEntry {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub ends_at: i64,
    pub scope: String,
    pub updated_at: i64,
    pub status: LimitEntryStatus,
}

impl LimitEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitEntryStatus) -> bool {
        match self.status {
            LimitEntryStatus::Planned => {
                match next {
                    LimitEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitEntryStatus::Scheduled => {
                return next == LimitEntryStatus::Completed;
            }
            LimitEntryStatus::InProgress => {
                match next {
                    LimitEntryStatus::Completed | LimitEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitEntryStatus::Completed => {
                match next {
                    LimitEntryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LimitEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "limit entry", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ends_at as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitEntry {
        LimitEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            ends_at: self.ends_at,
            scope: self.scope.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a limit entry in status planned.
pub fn new_limit_entry(code: &str, name: &str) -> LimitEntry {
    LimitEntry {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        ends_at: 0,
        scope: "".to_string(),
        updated_at: 0,
        status: LimitEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_limit_entry("code-275", "name-470");
        assert_eq!(item.status, LimitEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_limit_entry("code-275", "name-470");
        if item.move_to(LimitEntryStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitEntryStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_limit_entry("code-275", "name-470");
        item.status = LimitEntryStatus::Cancelled;
        assert!(
            item.move_to(LimitEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
