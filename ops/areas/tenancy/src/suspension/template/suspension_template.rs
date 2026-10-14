use super::*;

use acme_kit::text;

/// Where a suspension template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SuspensionTemplateStatus {
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

impl std::fmt::Display for SuspensionTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension template as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTemplate {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub capacity: i64,
    pub notes: String,
    pub revision: i64,
    pub source: String,
    pub reviewer: String,
    pub issuer: String,
    pub status: SuspensionTemplateStatus,
}

impl SuspensionTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionTemplateStatus) -> bool {
        match self.status {
            SuspensionTemplateStatus::Planned => {
                return next == SuspensionTemplateStatus::Completed;
            }
            SuspensionTemplateStatus::Scheduled => {
                match next {
                    SuspensionTemplateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionTemplateStatus::InProgress => {
                return next == SuspensionTemplateStatus::Completed;
            }
            SuspensionTemplateStatus::Completed => {
                return next == SuspensionTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.revision as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionTemplate {
        SuspensionTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            capacity: self.capacity,
            notes: self.notes.clone(),
            revision: self.revision,
            source: self.source.clone(),
            reviewer: self.reviewer.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a suspension template in status planned.
pub fn new_suspension_template(code: &str, title: &str) -> SuspensionTemplate {
    SuspensionTemplate {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        capacity: 0,
        notes: "".to_string(),
        revision: 0,
        source: "".to_string(),
        reviewer: "".to_string(),
        issuer: "".to_string(),
        status: SuspensionTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_suspension_template("code-171", "title-793");
        assert_eq!(item.status, SuspensionTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_suspension_template("code-171", "title-793");
        if item.move_to(SuspensionTemplateStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionTemplateStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_suspension_template("code-171", "title-793");
        item.status = SuspensionTemplateStatus::Cancelled;
        assert!(
            item.move_to(SuspensionTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
