use super::*;

use acme_kit::text;

/// Where a scheduled peering draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledPeeringDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledPeeringDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledPeeringDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled peering draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringDraft {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub currency: String,
    pub host: String,
    pub channel: String,
    pub scope: String,
    pub status: ScheduledPeeringDraftStatus,
}

impl ScheduledPeeringDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledPeeringDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledPeeringDraftStatus) -> bool {
        match self.status {
            ScheduledPeeringDraftStatus::Created => {
                match next {
                    ScheduledPeeringDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPeeringDraftStatus::Validated => {
                match next {
                    ScheduledPeeringDraftStatus::Published | ScheduledPeeringDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPeeringDraftStatus::Published => {
                return next == ScheduledPeeringDraftStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledPeeringDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled peering draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> ScheduledPeeringDraft {
        ScheduledPeeringDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            currency: self.currency.clone(),
            host: self.host.clone(),
            channel: self.channel.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a scheduled peering draft in status created.
pub fn new_scheduled_peering_draft(code: &str, title: &str) -> ScheduledPeeringDraft {
    ScheduledPeeringDraft {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        currency: "".to_string(),
        host: "".to_string(),
        channel: "".to_string(),
        scope: "".to_string(),
        status: ScheduledPeeringDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_peering_draft("code-342", "title-469");
        assert_eq!(item.status, ScheduledPeeringDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_scheduled_peering_draft("code-342", "title-469");
        if item.move_to(ScheduledPeeringDraftStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledPeeringDraftStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_peering_draft("code-342", "title-469");
        item.status = ScheduledPeeringDraftStatus::Retired;
        assert!(
            item.move_to(ScheduledPeeringDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
