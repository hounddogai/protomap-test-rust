use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a postmortem schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PostmortemScheduleStatus {
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

impl std::fmt::Display for PostmortemScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem schedule as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSchedule {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub country: String,
    pub attributes: BTreeMap<String, String>,
    pub quantity: i64,
    pub status: PostmortemScheduleStatus,
}

impl PostmortemSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemScheduleStatus) -> bool {
        match self.status {
            PostmortemScheduleStatus::Idle => {
                match next {
                    PostmortemScheduleStatus::Warming | PostmortemScheduleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemScheduleStatus::Warming => {
                return next == PostmortemScheduleStatus::Draining;
            }
            PostmortemScheduleStatus::Ready => {
                match next {
                    PostmortemScheduleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemScheduleStatus::Draining => {
                match next {
                    PostmortemScheduleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PostmortemScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> PostmortemSchedule {
        PostmortemSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            country: self.country.clone(),
            attributes: self.attributes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a postmortem schedule in status idle.
pub fn new_postmortem_schedule(code: &str, name: &str) -> PostmortemSchedule {
    PostmortemSchedule {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        country: "".to_string(),
        attributes: BTreeMap::new(),
        quantity: 0,
        status: PostmortemScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_postmortem_schedule("code-858", "name-299");
        assert_eq!(item.status, PostmortemScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_postmortem_schedule("code-858", "name-299");
        if item.move_to(PostmortemScheduleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemScheduleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_postmortem_schedule("code-858", "name-299");
        item.status = PostmortemScheduleStatus::Stopped;
        assert!(
            item.move_to(PostmortemScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
