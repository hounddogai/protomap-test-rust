use super::*;

use acme_kit::text;

/// Where a scheduled region pin profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRegionPinProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledRegionPinProfileStatus {
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

impl std::fmt::Display for ScheduledRegionPinProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled region pin profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinProfile {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub issuer: String,
    pub duration_minutes: i64,
    pub status: ScheduledRegionPinProfileStatus,
}

impl ScheduledRegionPinProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRegionPinProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRegionPinProfileStatus) -> bool {
        match self.status {
            ScheduledRegionPinProfileStatus::Draft => {
                match next {
                    ScheduledRegionPinProfileStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRegionPinProfileStatus::Pending => {
                match next {
                    ScheduledRegionPinProfileStatus::Active | ScheduledRegionPinProfileStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRegionPinProfileStatus::Active => {
                match next {
                    ScheduledRegionPinProfileStatus::Suspended | ScheduledRegionPinProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRegionPinProfileStatus::Suspended => {
                match next {
                    ScheduledRegionPinProfileStatus::Closed => {
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

    pub fn move_to(&mut self, next: ScheduledRegionPinProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled region pin profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> ScheduledRegionPinProfile {
        ScheduledRegionPinProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            issuer: self.issuer.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled region pin profile in status draft.
pub fn new_scheduled_region_pin_profile(code: &str, name: &str) -> ScheduledRegionPinProfile {
    ScheduledRegionPinProfile {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        issuer: "".to_string(),
        duration_minutes: 0,
        status: ScheduledRegionPinProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_region_pin_profile("code-943", "name-748");
        assert_eq!(item.status, ScheduledRegionPinProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_scheduled_region_pin_profile("code-943", "name-748");
        if item.move_to(ScheduledRegionPinProfileStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRegionPinProfileStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_region_pin_profile("code-943", "name-748");
        item.status = ScheduledRegionPinProfileStatus::Closed;
        assert!(
            item.move_to(ScheduledRegionPinProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
