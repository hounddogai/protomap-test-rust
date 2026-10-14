use super::*;

use acme_kit::text;

/// Where a redaction preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionPreferenceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RedactionPreferenceStatus {
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

impl std::fmt::Display for RedactionPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPreference {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub ceiling: f64,
    pub region: String,
    pub status: RedactionPreferenceStatus,
}

impl RedactionPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionPreferenceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionPreferenceStatus) -> bool {
        match self.status {
            RedactionPreferenceStatus::Planned => {
                return next == RedactionPreferenceStatus::Scheduled || next == RedactionPreferenceStatus::Completed;
            }
            RedactionPreferenceStatus::Scheduled => {
                return next == RedactionPreferenceStatus::InProgress || next == RedactionPreferenceStatus::Completed;
            }
            RedactionPreferenceStatus::InProgress => {
                return next == RedactionPreferenceStatus::Completed || next == RedactionPreferenceStatus::Cancelled;
            }
            RedactionPreferenceStatus::Completed => {
                match next {
                    RedactionPreferenceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RedactionPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.score * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionPreference {
        RedactionPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            ceiling: self.ceiling,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a redaction preference in status planned.
pub fn new_redaction_preference(code: &str, name: &str) -> RedactionPreference {
    RedactionPreference {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        ceiling: 0.0,
        region: "".to_string(),
        status: RedactionPreferenceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_redaction_preference("code-791", "name-264");
        assert_eq!(item.status, RedactionPreferenceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_redaction_preference("code-791", "name-264");
        if item.move_to(RedactionPreferenceStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionPreferenceStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_redaction_preference("code-791", "name-264");
        item.status = RedactionPreferenceStatus::Cancelled;
        assert!(
            item.move_to(RedactionPreferenceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
