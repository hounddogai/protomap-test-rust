use super::*;

use acme_kit::text;

/// Where a trust domain schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TrustDomainScheduleStatus {
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

impl std::fmt::Display for TrustDomainScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a trust domain schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSchedule {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub region: String,
    pub archived: bool,
    pub title: String,
    pub sequence: i64,
    pub phone: String,
    pub status: TrustDomainScheduleStatus,
}

impl TrustDomainSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainScheduleStatus) -> bool {
        match self.status {
            TrustDomainScheduleStatus::New => {
                return next == TrustDomainScheduleStatus::Queued;
            }
            TrustDomainScheduleStatus::Queued => {
                return next == TrustDomainScheduleStatus::Running;
            }
            TrustDomainScheduleStatus::Running => {
                return next == TrustDomainScheduleStatus::Done || next == TrustDomainScheduleStatus::Failed;
            }
            TrustDomainScheduleStatus::Done => {
                match next {
                    TrustDomainScheduleStatus::Failed => {
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

    pub fn move_to(&mut self, next: TrustDomainScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.sequence as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainSchedule {
        TrustDomainSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            region: self.region.clone(),
            archived: self.archived,
            title: self.title.clone(),
            sequence: self.sequence,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a trust domain schedule in status new.
pub fn new_trust_domain_schedule(code: &str, label: &str) -> TrustDomainSchedule {
    TrustDomainSchedule {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        region: "".to_string(),
        archived: false,
        title: "".to_string(),
        sequence: 0,
        phone: "".to_string(),
        status: TrustDomainScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_trust_domain_schedule("code-956", "label-480");
        assert_eq!(item.status, TrustDomainScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_trust_domain_schedule("code-956", "label-480");
        if item.move_to(TrustDomainScheduleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainScheduleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_trust_domain_schedule("code-956", "label-480");
        item.status = TrustDomainScheduleStatus::Failed;
        assert!(
            item.move_to(TrustDomainScheduleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
