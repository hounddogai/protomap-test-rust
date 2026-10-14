use super::*;

use acme_kit::text;

/// Where a regional egress schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalEgressScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalEgressScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RegionalEgressScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional egress schedule as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressSchedule {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub tags: Vec<String>,
    pub fingerprint: String,
    pub score: f64,
    pub status: RegionalEgressScheduleStatus,
}

impl RegionalEgressSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalEgressScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalEgressScheduleStatus) -> bool {
        match self.status {
            RegionalEgressScheduleStatus::Draft => {
                return next == RegionalEgressScheduleStatus::Suspended || next == RegionalEgressScheduleStatus::Closed;
            }
            RegionalEgressScheduleStatus::Pending => {
                match next {
                    RegionalEgressScheduleStatus::Suspended | RegionalEgressScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalEgressScheduleStatus::Active => {
                match next {
                    RegionalEgressScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalEgressScheduleStatus::Suspended => {
                return next == RegionalEgressScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalEgressScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional egress schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional egress schedule", self.code)
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
        self.score
    }

    pub fn copy(&self) -> RegionalEgressSchedule {
        RegionalEgressSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            tags: self.tags.clone(),
            fingerprint: self.fingerprint.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional egress schedule in status draft.
pub fn new_regional_egress_schedule(code: &str, title: &str) -> RegionalEgressSchedule {
    RegionalEgressSchedule {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        tags: Vec::new(),
        fingerprint: "".to_string(),
        score: 0.0,
        status: RegionalEgressScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_egress_schedule("code-192", "title-901");
        assert_eq!(item.status, RegionalEgressScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_regional_egress_schedule("code-192", "title-901");
        if item.move_to(RegionalEgressScheduleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalEgressScheduleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_egress_schedule("code-192", "title-901");
        item.status = RegionalEgressScheduleStatus::Closed;
        assert!(
            item.move_to(RegionalEgressScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
