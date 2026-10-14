use super::*;

use acme_kit::text;

/// Where a primary canary tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryCanaryTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryCanaryTierStatus {
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

impl std::fmt::Display for PrimaryCanaryTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary canary tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryTier {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub source: String,
    pub weight: f64,
    pub label: String,
    pub port: i64,
    pub status: PrimaryCanaryTierStatus,
}

impl PrimaryCanaryTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryCanaryTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryCanaryTierStatus) -> bool {
        match self.status {
            PrimaryCanaryTierStatus::Planned => {
                return next == PrimaryCanaryTierStatus::Scheduled || next == PrimaryCanaryTierStatus::InProgress;
            }
            PrimaryCanaryTierStatus::Scheduled => {
                return next == PrimaryCanaryTierStatus::Completed || next == PrimaryCanaryTierStatus::Cancelled;
            }
            PrimaryCanaryTierStatus::InProgress => {
                return next == PrimaryCanaryTierStatus::Completed || next == PrimaryCanaryTierStatus::Cancelled;
            }
            PrimaryCanaryTierStatus::Completed => {
                match next {
                    PrimaryCanaryTierStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimaryCanaryTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary canary tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.weight + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryCanaryTier {
        PrimaryCanaryTier {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            source: self.source.clone(),
            weight: self.weight,
            label: self.label.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a primary canary tier in status planned.
pub fn new_primary_canary_tier(code: &str, title: &str) -> PrimaryCanaryTier {
    PrimaryCanaryTier {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        source: "".to_string(),
        weight: 0.0,
        label: "".to_string(),
        port: 0,
        status: PrimaryCanaryTierStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_canary_tier("code-888", "title-720");
        assert_eq!(item.status, PrimaryCanaryTierStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_primary_canary_tier("code-888", "title-720");
        if item.move_to(PrimaryCanaryTierStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryCanaryTierStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_canary_tier("code-888", "title-720");
        item.status = PrimaryCanaryTierStatus::Cancelled;
        assert!(
            item.move_to(PrimaryCanaryTierStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
