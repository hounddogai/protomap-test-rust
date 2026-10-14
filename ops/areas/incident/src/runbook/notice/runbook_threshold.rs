use super::*;

use acme_kit::text;

/// Where a runbook threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookThresholdStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RunbookThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RunbookThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookThreshold {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub capacity: i64,
    pub host: String,
    pub maximum: f64,
    pub pinned: bool,
    pub ends_at: i64,
    pub external_ref: String,
    pub locale: String,
    pub cipher: String,
    pub status: RunbookThresholdStatus,
}

impl RunbookThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookThresholdStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookThresholdStatus) -> bool {
        match self.status {
            RunbookThresholdStatus::Created => {
                return next == RunbookThresholdStatus::Published || next == RunbookThresholdStatus::Retired;
            }
            RunbookThresholdStatus::Validated => {
                return next == RunbookThresholdStatus::Published;
            }
            RunbookThresholdStatus::Published => {
                return next == RunbookThresholdStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "runbook threshold", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.expires_at as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookThreshold {
        RunbookThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            capacity: self.capacity,
            host: self.host.clone(),
            maximum: self.maximum,
            pinned: self.pinned,
            ends_at: self.ends_at,
            external_ref: self.external_ref.clone(),
            locale: self.locale.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a runbook threshold in status created.
pub fn new_runbook_threshold(code: &str, name: &str) -> RunbookThreshold {
    RunbookThreshold {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        capacity: 0,
        host: "".to_string(),
        maximum: 0.0,
        pinned: false,
        ends_at: 0,
        external_ref: "".to_string(),
        locale: "".to_string(),
        cipher: "".to_string(),
        status: RunbookThresholdStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_runbook_threshold("code-102", "name-649");
        assert_eq!(item.status, RunbookThresholdStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_runbook_threshold("code-102", "name-649");
        if item.move_to(RunbookThresholdStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookThresholdStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_runbook_threshold("code-102", "name-649");
        item.status = RunbookThresholdStatus::Retired;
        assert!(
            item.move_to(RunbookThresholdStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
