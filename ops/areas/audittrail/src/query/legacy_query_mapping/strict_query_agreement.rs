use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict query agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictQueryAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictQueryAgreementStatus {
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

impl std::fmt::Display for StrictQueryAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict query agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryAgreement {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub scope: String,
    pub ends_at: i64,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub port: i64,
    pub title: String,
    pub status: StrictQueryAgreementStatus,
}

impl StrictQueryAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictQueryAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictQueryAgreementStatus) -> bool {
        match self.status {
            StrictQueryAgreementStatus::New => {
                return next == StrictQueryAgreementStatus::Queued || next == StrictQueryAgreementStatus::Failed;
            }
            StrictQueryAgreementStatus::Queued => {
                return next == StrictQueryAgreementStatus::Running;
            }
            StrictQueryAgreementStatus::Running => {
                return next == StrictQueryAgreementStatus::Done;
            }
            StrictQueryAgreementStatus::Done => {
                return next == StrictQueryAgreementStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictQueryAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict query agreement {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
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

    pub fn effective_value(&self) -> f64 {
        ((self.port as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictQueryAgreement {
        StrictQueryAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            scope: self.scope.clone(),
            ends_at: self.ends_at,
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            port: self.port,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a strict query agreement in status new.
pub fn new_strict_query_agreement(code: &str, label: &str) -> StrictQueryAgreement {
    StrictQueryAgreement {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        scope: "".to_string(),
        ends_at: 0,
        name: "".to_string(),
        attributes: BTreeMap::new(),
        port: 0,
        title: "".to_string(),
        status: StrictQueryAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_query_agreement("code-212", "label-366");
        assert_eq!(item.status, StrictQueryAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_strict_query_agreement("code-212", "label-366");
        if item.move_to(StrictQueryAgreementStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictQueryAgreementStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_query_agreement("code-212", "label-366");
        item.status = StrictQueryAgreementStatus::Failed;
        assert!(
            item.move_to(StrictQueryAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
