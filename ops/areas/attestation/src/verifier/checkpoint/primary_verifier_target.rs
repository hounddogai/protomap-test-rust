use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary verifier target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryVerifierTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryVerifierTargetStatus {
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

impl std::fmt::Display for PrimaryVerifierTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary verifier target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTarget {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub capacity: i64,
    pub tags: Vec<String>,
    pub retries: i64,
    pub attributes: BTreeMap<String, String>,
    pub priority: i64,
    pub currency: String,
    pub region: String,
    pub port: i64,
    pub status: PrimaryVerifierTargetStatus,
}

impl PrimaryVerifierTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryVerifierTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryVerifierTargetStatus) -> bool {
        match self.status {
            PrimaryVerifierTargetStatus::New => {
                return next == PrimaryVerifierTargetStatus::Queued || next == PrimaryVerifierTargetStatus::Running;
            }
            PrimaryVerifierTargetStatus::Queued => {
                return next == PrimaryVerifierTargetStatus::Running || next == PrimaryVerifierTargetStatus::Failed;
            }
            PrimaryVerifierTargetStatus::Running => {
                return next == PrimaryVerifierTargetStatus::Done;
            }
            PrimaryVerifierTargetStatus::Done => {
                match next {
                    PrimaryVerifierTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimaryVerifierTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary verifier target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn adjusted_load(&self) -> f64 {
        ((self.priority as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryVerifierTarget {
        PrimaryVerifierTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            capacity: self.capacity,
            tags: self.tags.clone(),
            retries: self.retries,
            attributes: self.attributes.clone(),
            priority: self.priority,
            currency: self.currency.clone(),
            region: self.region.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary verifier target in status new.
pub fn new_primary_verifier_target(code: &str, name: &str) -> PrimaryVerifierTarget {
    PrimaryVerifierTarget {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        capacity: 0,
        tags: Vec::new(),
        retries: 0,
        attributes: BTreeMap::new(),
        priority: 0,
        currency: "".to_string(),
        region: "".to_string(),
        port: 0,
        status: PrimaryVerifierTargetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_verifier_target("code-657", "name-40");
        assert_eq!(item.status, PrimaryVerifierTargetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_primary_verifier_target("code-657", "name-40");
        if item.move_to(PrimaryVerifierTargetStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryVerifierTargetStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_verifier_target("code-657", "name-40");
        item.status = PrimaryVerifierTargetStatus::Failed;
        assert!(
            item.move_to(PrimaryVerifierTargetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
