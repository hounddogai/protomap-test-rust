use super::*;

use acme_kit::text;

/// Where a regional postmortem agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPostmortemAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalPostmortemAgreementStatus {
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

impl std::fmt::Display for RegionalPostmortemAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional postmortem agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemAgreement {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub tags: Vec<String>,
    pub updated_at: i64,
    pub duration_minutes: i64,
    pub expires_at: i64,
    pub rotation_days: i64,
    pub name: String,
    pub status: RegionalPostmortemAgreementStatus,
}

impl RegionalPostmortemAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPostmortemAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPostmortemAgreementStatus) -> bool {
        match self.status {
            RegionalPostmortemAgreementStatus::New => {
                return next == RegionalPostmortemAgreementStatus::Queued
                    || next == RegionalPostmortemAgreementStatus::Done;
            }
            RegionalPostmortemAgreementStatus::Queued => {
                return next == RegionalPostmortemAgreementStatus::Running
                    || next == RegionalPostmortemAgreementStatus::Done;
            }
            RegionalPostmortemAgreementStatus::Running => {
                return next == RegionalPostmortemAgreementStatus::Done
                    || next == RegionalPostmortemAgreementStatus::Failed;
            }
            RegionalPostmortemAgreementStatus::Done => {
                match next {
                    RegionalPostmortemAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionalPostmortemAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional postmortem agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional postmortem agreement", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.updated_at as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPostmortemAgreement {
        RegionalPostmortemAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            tags: self.tags.clone(),
            updated_at: self.updated_at,
            duration_minutes: self.duration_minutes,
            expires_at: self.expires_at,
            rotation_days: self.rotation_days,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional postmortem agreement in status new.
pub fn new_regional_postmortem_agreement(code: &str, label: &str) -> RegionalPostmortemAgreement {
    RegionalPostmortemAgreement {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        tags: Vec::new(),
        updated_at: 0,
        duration_minutes: 0,
        expires_at: 0,
        rotation_days: 0,
        name: "".to_string(),
        status: RegionalPostmortemAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_postmortem_agreement("code-102", "label-765");
        assert_eq!(item.status, RegionalPostmortemAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_regional_postmortem_agreement("code-102", "label-765");
        if item.move_to(RegionalPostmortemAgreementStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalPostmortemAgreementStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_postmortem_agreement("code-102", "label-765");
        item.status = RegionalPostmortemAgreementStatus::Failed;
        assert!(
            item.move_to(RegionalPostmortemAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
