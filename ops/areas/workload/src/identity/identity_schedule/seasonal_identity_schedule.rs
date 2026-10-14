use super::*;

use acme_kit::text;

/// Where a seasonal identity schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalIdentityScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalIdentityScheduleStatus {
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

impl std::fmt::Display for SeasonalIdentityScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal identity schedule as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentitySchedule {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub minimum: f64,
    pub ratio: f64,
    pub archived: bool,
    pub batch_size: i64,
    pub tags: Vec<String>,
    pub approved: bool,
    pub checksum: String,
    pub status: SeasonalIdentityScheduleStatus,
}

impl SeasonalIdentitySchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalIdentityScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalIdentityScheduleStatus) -> bool {
        match self.status {
            SeasonalIdentityScheduleStatus::New => {
                match next {
                    SeasonalIdentityScheduleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalIdentityScheduleStatus::Queued => {
                return next == SeasonalIdentityScheduleStatus::Running
                    || next == SeasonalIdentityScheduleStatus::Failed;
            }
            SeasonalIdentityScheduleStatus::Running => {
                match next {
                    SeasonalIdentityScheduleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalIdentityScheduleStatus::Done => {
                return next == SeasonalIdentityScheduleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalIdentityScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal identity schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.minimum - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalIdentitySchedule {
        SeasonalIdentitySchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            minimum: self.minimum,
            ratio: self.ratio,
            archived: self.archived,
            batch_size: self.batch_size,
            tags: self.tags.clone(),
            approved: self.approved,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal identity schedule in status new.
pub fn new_seasonal_identity_schedule(code: &str, label: &str) -> SeasonalIdentitySchedule {
    SeasonalIdentitySchedule {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        minimum: 0.0,
        ratio: 0.0,
        archived: false,
        batch_size: 0,
        tags: Vec::new(),
        approved: false,
        checksum: "".to_string(),
        status: SeasonalIdentityScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_identity_schedule("code-497", "label-352");
        assert_eq!(item.status, SeasonalIdentityScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_seasonal_identity_schedule("code-497", "label-352");
        if item.move_to(SeasonalIdentityScheduleStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalIdentityScheduleStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_identity_schedule("code-497", "label-352");
        item.status = SeasonalIdentityScheduleStatus::Failed;
        assert!(
            item.move_to(SeasonalIdentityScheduleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
