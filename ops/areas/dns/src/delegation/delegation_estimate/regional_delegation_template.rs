use super::*;

use acme_kit::text;

/// Where a regional delegation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDelegationTemplateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalDelegationTemplateStatus {
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

impl std::fmt::Display for RegionalDelegationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional delegation template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationTemplate {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub ends_at: i64,
    pub created_at: i64,
    pub segment: String,
    pub status: RegionalDelegationTemplateStatus,
}

impl RegionalDelegationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDelegationTemplateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDelegationTemplateStatus) -> bool {
        match self.status {
            RegionalDelegationTemplateStatus::New => {
                return next == RegionalDelegationTemplateStatus::Done;
            }
            RegionalDelegationTemplateStatus::Queued => {
                return next == RegionalDelegationTemplateStatus::Done
                    || next == RegionalDelegationTemplateStatus::Failed;
            }
            RegionalDelegationTemplateStatus::Running => {
                return next == RegionalDelegationTemplateStatus::Done
                    || next == RegionalDelegationTemplateStatus::Failed;
            }
            RegionalDelegationTemplateStatus::Done => {
                return next == RegionalDelegationTemplateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalDelegationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional delegation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ends_at as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDelegationTemplate {
        RegionalDelegationTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            ends_at: self.ends_at,
            created_at: self.created_at,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional delegation template in status new.
pub fn new_regional_delegation_template(code: &str, name: &str) -> RegionalDelegationTemplate {
    RegionalDelegationTemplate {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        ends_at: 0,
        created_at: 0,
        segment: "".to_string(),
        status: RegionalDelegationTemplateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_delegation_template("code-897", "name-905");
        assert_eq!(item.status, RegionalDelegationTemplateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_regional_delegation_template("code-897", "name-905");
        if item.move_to(RegionalDelegationTemplateStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalDelegationTemplateStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_delegation_template("code-897", "name-905");
        item.status = RegionalDelegationTemplateStatus::Failed;
        assert!(
            item.move_to(RegionalDelegationTemplateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
