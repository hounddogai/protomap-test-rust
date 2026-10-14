use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a overage policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OveragePolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl OveragePolicyStatus {
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

impl std::fmt::Display for OveragePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a overage policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct OveragePolicy {
    pub code: String,
    pub label: String,
    pub country: String,
    pub attributes: BTreeMap<String, String>,
    pub budget: f64,
    pub status: OveragePolicyStatus,
}

impl OveragePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OveragePolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OveragePolicyStatus) -> bool {
        match self.status {
            OveragePolicyStatus::New => {
                match next {
                    OveragePolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OveragePolicyStatus::Queued => {
                return next == OveragePolicyStatus::Failed;
            }
            OveragePolicyStatus::Running => {
                match next {
                    OveragePolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OveragePolicyStatus::Done => {
                return next == OveragePolicyStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OveragePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "overage policy", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> OveragePolicy {
        OveragePolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            attributes: self.attributes.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a overage policy in status new.
pub fn new_overage_policy(code: &str, label: &str) -> OveragePolicy {
    OveragePolicy {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        attributes: BTreeMap::new(),
        budget: 0.0,
        status: OveragePolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_overage_policy("code-748", "label-955");
        assert_eq!(item.status, OveragePolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_overage_policy("code-748", "label-955");
        if item.move_to(OveragePolicyStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OveragePolicyStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_overage_policy("code-748", "label-955");
        item.status = OveragePolicyStatus::Failed;
        assert!(item.move_to(OveragePolicyStatus::New).is_err(), "expected move to fail");
    }
}
