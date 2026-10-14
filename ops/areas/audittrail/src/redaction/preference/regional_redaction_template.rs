use super::*;

use acme_kit::text;

/// Where a regional redaction template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRedactionTemplateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalRedactionTemplateStatus {
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

impl std::fmt::Display for RegionalRedactionTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional redaction template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionTemplate {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub external_ref: String,
    pub minimum: f64,
    pub priority: i64,
    pub fingerprint: String,
    pub quantity: i64,
    pub floor_value: f64,
    pub burst: i64,
    pub tenant_id: String,
    pub status: RegionalRedactionTemplateStatus,
}

impl RegionalRedactionTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRedactionTemplateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRedactionTemplateStatus) -> bool {
        match self.status {
            RegionalRedactionTemplateStatus::New => {
                match next {
                    RegionalRedactionTemplateStatus::Done | RegionalRedactionTemplateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRedactionTemplateStatus::Queued => {
                match next {
                    RegionalRedactionTemplateStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRedactionTemplateStatus::Running => {
                return next == RegionalRedactionTemplateStatus::Done;
            }
            RegionalRedactionTemplateStatus::Done => {
                match next {
                    RegionalRedactionTemplateStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionalRedactionTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional redaction template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional redaction template", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.sequence as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRedactionTemplate {
        RegionalRedactionTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            external_ref: self.external_ref.clone(),
            minimum: self.minimum,
            priority: self.priority,
            fingerprint: self.fingerprint.clone(),
            quantity: self.quantity,
            floor_value: self.floor_value,
            burst: self.burst,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional redaction template in status new.
pub fn new_regional_redaction_template(code: &str, title: &str) -> RegionalRedactionTemplate {
    RegionalRedactionTemplate {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        external_ref: "".to_string(),
        minimum: 0.0,
        priority: 0,
        fingerprint: "".to_string(),
        quantity: 0,
        floor_value: 0.0,
        burst: 0,
        tenant_id: "".to_string(),
        status: RegionalRedactionTemplateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_redaction_template("code-81", "title-691");
        assert_eq!(item.status, RegionalRedactionTemplateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_regional_redaction_template("code-81", "title-691");
        if item.move_to(RegionalRedactionTemplateStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRedactionTemplateStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_redaction_template("code-81", "title-691");
        item.status = RegionalRedactionTemplateStatus::Failed;
        assert!(
            item.move_to(RegionalRedactionTemplateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
