use super::*;

use acme_kit::text;

/// Where a regional failover override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFailoverOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalFailoverOverrideStatus {
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

impl std::fmt::Display for RegionalFailoverOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional failover override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverOverride {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub minimum: f64,
    pub checksum: String,
    pub locale: String,
    pub status: RegionalFailoverOverrideStatus,
}

impl RegionalFailoverOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFailoverOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFailoverOverrideStatus) -> bool {
        match self.status {
            RegionalFailoverOverrideStatus::New => {
                return next == RegionalFailoverOverrideStatus::Running;
            }
            RegionalFailoverOverrideStatus::Queued => {
                match next {
                    RegionalFailoverOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFailoverOverrideStatus::Running => {
                match next {
                    RegionalFailoverOverrideStatus::Done | RegionalFailoverOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFailoverOverrideStatus::Done => {
                return next == RegionalFailoverOverrideStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalFailoverOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional failover override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional failover override", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> RegionalFailoverOverride {
        RegionalFailoverOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            minimum: self.minimum,
            checksum: self.checksum.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a regional failover override in status new.
pub fn new_regional_failover_override(code: &str, title: &str) -> RegionalFailoverOverride {
    RegionalFailoverOverride {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        minimum: 0.0,
        checksum: "".to_string(),
        locale: "".to_string(),
        status: RegionalFailoverOverrideStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_failover_override("code-59", "title-785");
        assert_eq!(item.status, RegionalFailoverOverrideStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_regional_failover_override("code-59", "title-785");
        if item.move_to(RegionalFailoverOverrideStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalFailoverOverrideStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_failover_override("code-59", "title-785");
        item.status = RegionalFailoverOverrideStatus::Failed;
        assert!(
            item.move_to(RegionalFailoverOverrideStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
