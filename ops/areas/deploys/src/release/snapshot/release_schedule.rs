use super::*;

use acme_kit::text;

/// Where a release schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ReleaseScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ReleaseScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSchedule {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub host: String,
    pub locale: String,
    pub channel: String,
    pub tags: Vec<String>,
    pub name: String,
    pub status: ReleaseScheduleStatus,
}

impl ReleaseSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseScheduleStatus) -> bool {
        match self.status {
            ReleaseScheduleStatus::Proposed => {
                match next {
                    ReleaseScheduleStatus::Approved | ReleaseScheduleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseScheduleStatus::Approved => {
                match next {
                    ReleaseScheduleStatus::Rejected | ReleaseScheduleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseScheduleStatus::Rejected => {
                match next {
                    ReleaseScheduleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ReleaseScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> ReleaseSchedule {
        ReleaseSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            host: self.host.clone(),
            locale: self.locale.clone(),
            channel: self.channel.clone(),
            tags: self.tags.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a release schedule in status proposed.
pub fn new_release_schedule(code: &str, label: &str) -> ReleaseSchedule {
    ReleaseSchedule {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        host: "".to_string(),
        locale: "".to_string(),
        channel: "".to_string(),
        tags: Vec::new(),
        name: "".to_string(),
        status: ReleaseScheduleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_release_schedule("code-242", "label-729");
        assert_eq!(item.status, ReleaseScheduleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_release_schedule("code-242", "label-729");
        if item.move_to(ReleaseScheduleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseScheduleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_release_schedule("code-242", "label-729");
        item.status = ReleaseScheduleStatus::Withdrawn;
        assert!(
            item.move_to(ReleaseScheduleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
