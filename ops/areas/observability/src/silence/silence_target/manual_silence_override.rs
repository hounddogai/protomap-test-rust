use super::*;

use acme_kit::text;

/// Where a manual silence override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSilenceOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualSilenceOverrideStatus {
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

impl std::fmt::Display for ManualSilenceOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual silence override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceOverride {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub category: String,
    pub key_id: String,
    pub summary: String,
    pub notes: String,
    pub cost: f64,
    pub ttl_seconds: i64,
    pub sequence: i64,
    pub status: ManualSilenceOverrideStatus,
}

impl ManualSilenceOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSilenceOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSilenceOverrideStatus) -> bool {
        match self.status {
            ManualSilenceOverrideStatus::New => {
                match next {
                    ManualSilenceOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSilenceOverrideStatus::Queued => {
                match next {
                    ManualSilenceOverrideStatus::Running | ManualSilenceOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSilenceOverrideStatus::Running => {
                return next == ManualSilenceOverrideStatus::Failed;
            }
            ManualSilenceOverrideStatus::Done => {
                match next {
                    ManualSilenceOverrideStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualSilenceOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual silence override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual silence override", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.sequence as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualSilenceOverride {
        ManualSilenceOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            category: self.category.clone(),
            key_id: self.key_id.clone(),
            summary: self.summary.clone(),
            notes: self.notes.clone(),
            cost: self.cost,
            ttl_seconds: self.ttl_seconds,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a manual silence override in status new.
pub fn new_manual_silence_override(code: &str, name: &str) -> ManualSilenceOverride {
    ManualSilenceOverride {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        category: "".to_string(),
        key_id: "".to_string(),
        summary: "".to_string(),
        notes: "".to_string(),
        cost: 0.0,
        ttl_seconds: 0,
        sequence: 0,
        status: ManualSilenceOverrideStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_silence_override("code-943", "name-891");
        assert_eq!(item.status, ManualSilenceOverrideStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_manual_silence_override("code-943", "name-891");
        if item.move_to(ManualSilenceOverrideStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSilenceOverrideStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_silence_override("code-943", "name-891");
        item.status = ManualSilenceOverrideStatus::Failed;
        assert!(
            item.move_to(ManualSilenceOverrideStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
