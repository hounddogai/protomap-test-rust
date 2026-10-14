use super::*;

use acme_kit::text;

/// Where a strict escalation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictEscalationProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictEscalationProfileStatus {
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

impl std::fmt::Display for StrictEscalationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict escalation profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationProfile {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub ttl_seconds: i64,
    pub region: String,
    pub cluster: String,
    pub burst: i64,
    pub minimum: f64,
    pub status: StrictEscalationProfileStatus,
}

impl StrictEscalationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictEscalationProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictEscalationProfileStatus) -> bool {
        match self.status {
            StrictEscalationProfileStatus::New => {
                return next == StrictEscalationProfileStatus::Done;
            }
            StrictEscalationProfileStatus::Queued => {
                return next == StrictEscalationProfileStatus::Running || next == StrictEscalationProfileStatus::Done;
            }
            StrictEscalationProfileStatus::Running => {
                return next == StrictEscalationProfileStatus::Failed;
            }
            StrictEscalationProfileStatus::Done => {
                return next == StrictEscalationProfileStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictEscalationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict escalation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.minimum + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictEscalationProfile {
        StrictEscalationProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            ttl_seconds: self.ttl_seconds,
            region: self.region.clone(),
            cluster: self.cluster.clone(),
            burst: self.burst,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a strict escalation profile in status new.
pub fn new_strict_escalation_profile(code: &str, name: &str) -> StrictEscalationProfile {
    StrictEscalationProfile {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        ttl_seconds: 0,
        region: "".to_string(),
        cluster: "".to_string(),
        burst: 0,
        minimum: 0.0,
        status: StrictEscalationProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_escalation_profile("code-411", "name-796");
        assert_eq!(item.status, StrictEscalationProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_strict_escalation_profile("code-411", "name-796");
        if item.move_to(StrictEscalationProfileStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictEscalationProfileStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_escalation_profile("code-411", "name-796");
        item.status = StrictEscalationProfileStatus::Failed;
        assert!(
            item.move_to(StrictEscalationProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
