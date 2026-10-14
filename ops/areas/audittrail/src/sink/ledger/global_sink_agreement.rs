use super::*;

use acme_kit::text;

/// Where a global sink agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSinkAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalSinkAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalSinkAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global sink agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkAgreement {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub maximum: f64,
    pub score: f64,
    pub batch_size: i64,
    pub audience: String,
    pub grace_days: i64,
    pub minimum: f64,
    pub tags: Vec<String>,
    pub country: String,
    pub status: GlobalSinkAgreementStatus,
}

impl GlobalSinkAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSinkAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSinkAgreementStatus) -> bool {
        match self.status {
            GlobalSinkAgreementStatus::Created => {
                return next == GlobalSinkAgreementStatus::Validated || next == GlobalSinkAgreementStatus::Retired;
            }
            GlobalSinkAgreementStatus::Validated => {
                match next {
                    GlobalSinkAgreementStatus::Published | GlobalSinkAgreementStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSinkAgreementStatus::Published => {
                return next == GlobalSinkAgreementStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSinkAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global sink agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global sink agreement", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.minimum + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSinkAgreement {
        GlobalSinkAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            maximum: self.maximum,
            score: self.score,
            batch_size: self.batch_size,
            audience: self.audience.clone(),
            grace_days: self.grace_days,
            minimum: self.minimum,
            tags: self.tags.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global sink agreement in status created.
pub fn new_global_sink_agreement(code: &str, name: &str) -> GlobalSinkAgreement {
    GlobalSinkAgreement {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        maximum: 0.0,
        score: 0.0,
        batch_size: 0,
        audience: "".to_string(),
        grace_days: 0,
        minimum: 0.0,
        tags: Vec::new(),
        country: "".to_string(),
        status: GlobalSinkAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_sink_agreement("code-731", "name-912");
        assert_eq!(item.status, GlobalSinkAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_global_sink_agreement("code-731", "name-912");
        if item.move_to(GlobalSinkAgreementStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSinkAgreementStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_sink_agreement("code-731", "name-912");
        item.status = GlobalSinkAgreementStatus::Retired;
        assert!(
            item.move_to(GlobalSinkAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
