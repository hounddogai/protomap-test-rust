use super::*;

use acme_kit::text;

/// Where a postmortem cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PostmortemCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PostmortemCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCycle {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub window_days: i64,
    pub cost: f64,
    pub minimum: f64,
    pub quantity: i64,
    pub capacity: i64,
    pub status: PostmortemCycleStatus,
}

impl PostmortemCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemCycleStatus) -> bool {
        match self.status {
            PostmortemCycleStatus::Created => {
                match next {
                    PostmortemCycleStatus::Published | PostmortemCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemCycleStatus::Validated => {
                match next {
                    PostmortemCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemCycleStatus::Published => {
                return next == PostmortemCycleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem cycle", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.cost + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemCycle {
        PostmortemCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            window_days: self.window_days,
            cost: self.cost,
            minimum: self.minimum,
            quantity: self.quantity,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a postmortem cycle in status created.
pub fn new_postmortem_cycle(code: &str, label: &str) -> PostmortemCycle {
    PostmortemCycle {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        window_days: 0,
        cost: 0.0,
        minimum: 0.0,
        quantity: 0,
        capacity: 0,
        status: PostmortemCycleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_postmortem_cycle("code-973", "label-768");
        assert_eq!(item.status, PostmortemCycleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_postmortem_cycle("code-973", "label-768");
        if item.move_to(PostmortemCycleStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemCycleStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_postmortem_cycle("code-973", "label-768");
        item.status = PostmortemCycleStatus::Retired;
        assert!(
            item.move_to(PostmortemCycleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
