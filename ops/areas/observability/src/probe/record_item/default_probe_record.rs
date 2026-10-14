use super::*;

use acme_kit::text;

/// Where a default probe record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultProbeRecordStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultProbeRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for DefaultProbeRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default probe record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProbeRecord {
    pub code: String,
    pub label: String,
    pub country: String,
    pub revision: i64,
    pub created_at: i64,
    pub sequence: i64,
    pub retries: i64,
    pub namespace: String,
    pub tags: Vec<String>,
    pub grace_days: i64,
    pub status: DefaultProbeRecordStatus,
}

impl DefaultProbeRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultProbeRecordStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultProbeRecordStatus) -> bool {
        match self.status {
            DefaultProbeRecordStatus::Idle => {
                match next {
                    DefaultProbeRecordStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultProbeRecordStatus::Warming => {
                return next == DefaultProbeRecordStatus::Ready || next == DefaultProbeRecordStatus::Draining;
            }
            DefaultProbeRecordStatus::Ready => {
                return next == DefaultProbeRecordStatus::Draining || next == DefaultProbeRecordStatus::Stopped;
            }
            DefaultProbeRecordStatus::Draining => {
                match next {
                    DefaultProbeRecordStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultProbeRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default probe record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default probe record", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.created_at as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultProbeRecord {
        DefaultProbeRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            revision: self.revision,
            created_at: self.created_at,
            sequence: self.sequence,
            retries: self.retries,
            namespace: self.namespace.clone(),
            tags: self.tags.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a default probe record in status idle.
pub fn new_default_probe_record(code: &str, label: &str) -> DefaultProbeRecord {
    DefaultProbeRecord {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        revision: 0,
        created_at: 0,
        sequence: 0,
        retries: 0,
        namespace: "".to_string(),
        tags: Vec::new(),
        grace_days: 0,
        status: DefaultProbeRecordStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_probe_record("code-263", "label-116");
        assert_eq!(item.status, DefaultProbeRecordStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_default_probe_record("code-263", "label-116");
        if item.move_to(DefaultProbeRecordStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultProbeRecordStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_probe_record("code-263", "label-116");
        item.status = DefaultProbeRecordStatus::Stopped;
        assert!(
            item.move_to(DefaultProbeRecordStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
