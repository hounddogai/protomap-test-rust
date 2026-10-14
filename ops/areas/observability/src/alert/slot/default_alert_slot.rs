use super::*;

use acme_kit::text;

/// Where a default alert slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultAlertSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultAlertSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultAlertSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default alert slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertSlot {
    pub code: String,
    pub name: String,
    pub category: String,
    pub margin: f64,
    pub notes: String,
    pub starts_at: i64,
    pub ends_at: i64,
    pub burst: i64,
    pub tags: Vec<String>,
    pub status: DefaultAlertSlotStatus,
}

impl DefaultAlertSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultAlertSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultAlertSlotStatus) -> bool {
        match self.status {
            DefaultAlertSlotStatus::Proposed => {
                match next {
                    DefaultAlertSlotStatus::Approved | DefaultAlertSlotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultAlertSlotStatus::Approved => {
                return next == DefaultAlertSlotStatus::Withdrawn;
            }
            DefaultAlertSlotStatus::Rejected => {
                return next == DefaultAlertSlotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultAlertSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default alert slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.starts_at as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultAlertSlot {
        DefaultAlertSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            margin: self.margin,
            notes: self.notes.clone(),
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            burst: self.burst,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a default alert slot in status proposed.
pub fn new_default_alert_slot(code: &str, name: &str) -> DefaultAlertSlot {
    DefaultAlertSlot {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        margin: 0.0,
        notes: "".to_string(),
        starts_at: 0,
        ends_at: 0,
        burst: 0,
        tags: Vec::new(),
        status: DefaultAlertSlotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_alert_slot("code-815", "name-348");
        assert_eq!(item.status, DefaultAlertSlotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_default_alert_slot("code-815", "name-348");
        if item.move_to(DefaultAlertSlotStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultAlertSlotStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_alert_slot("code-815", "name-348");
        item.status = DefaultAlertSlotStatus::Withdrawn;
        assert!(
            item.move_to(DefaultAlertSlotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
