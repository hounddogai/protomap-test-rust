use super::*;

use acme_kit::text;

/// Where a adaptive runbook entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRunbookEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveRunbookEntryStatus {
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

impl std::fmt::Display for AdaptiveRunbookEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive runbook entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookEntry {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub rotation_days: i64,
    pub limit: i64,
    pub ratio: f64,
    pub maximum: f64,
    pub namespace: String,
    pub created_at: i64,
    pub external_ref: String,
    pub floor_value: f64,
    pub status: AdaptiveRunbookEntryStatus,
}

impl AdaptiveRunbookEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRunbookEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRunbookEntryStatus) -> bool {
        match self.status {
            AdaptiveRunbookEntryStatus::Idle => {
                match next {
                    AdaptiveRunbookEntryStatus::Draining | AdaptiveRunbookEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRunbookEntryStatus::Warming => {
                match next {
                    AdaptiveRunbookEntryStatus::Draining | AdaptiveRunbookEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRunbookEntryStatus::Ready => {
                return next == AdaptiveRunbookEntryStatus::Draining || next == AdaptiveRunbookEntryStatus::Stopped;
            }
            AdaptiveRunbookEntryStatus::Draining => {
                return next == AdaptiveRunbookEntryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRunbookEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive runbook entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.limit as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRunbookEntry {
        AdaptiveRunbookEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            rotation_days: self.rotation_days,
            limit: self.limit,
            ratio: self.ratio,
            maximum: self.maximum,
            namespace: self.namespace.clone(),
            created_at: self.created_at,
            external_ref: self.external_ref.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive runbook entry in status idle.
pub fn new_adaptive_runbook_entry(code: &str, title: &str) -> AdaptiveRunbookEntry {
    AdaptiveRunbookEntry {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        rotation_days: 0,
        limit: 0,
        ratio: 0.0,
        maximum: 0.0,
        namespace: "".to_string(),
        created_at: 0,
        external_ref: "".to_string(),
        floor_value: 0.0,
        status: AdaptiveRunbookEntryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_runbook_entry("code-140", "title-576");
        assert_eq!(item.status, AdaptiveRunbookEntryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_adaptive_runbook_entry("code-140", "title-576");
        if item.move_to(AdaptiveRunbookEntryStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRunbookEntryStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_runbook_entry("code-140", "title-576");
        item.status = AdaptiveRunbookEntryStatus::Stopped;
        assert!(
            item.move_to(AdaptiveRunbookEntryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
