use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient plan score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPlanScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientPlanScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientPlanScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient plan score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanScore {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub quota_units: i64,
    pub quantity: i64,
    pub sequence: i64,
    pub attributes: BTreeMap<String, String>,
    pub contact_email: String,
    pub status: LenientPlanScoreStatus,
}

impl LenientPlanScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPlanScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPlanScoreStatus) -> bool {
        match self.status {
            LenientPlanScoreStatus::Requested => {
                match next {
                    LenientPlanScoreStatus::Revoked | LenientPlanScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPlanScoreStatus::Granted => {
                match next {
                    LenientPlanScoreStatus::Revoked | LenientPlanScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPlanScoreStatus::Revoked => {
                match next {
                    LenientPlanScoreStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientPlanScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient plan score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient plan score", self.code)
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

    pub fn effective_load(&self) -> f64 {
        ((self.quota_units as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPlanScore {
        LenientPlanScore {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            quota_units: self.quota_units,
            quantity: self.quantity,
            sequence: self.sequence,
            attributes: self.attributes.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient plan score in status requested.
pub fn new_lenient_plan_score(code: &str, name: &str) -> LenientPlanScore {
    LenientPlanScore {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        quota_units: 0,
        quantity: 0,
        sequence: 0,
        attributes: BTreeMap::new(),
        contact_email: "".to_string(),
        status: LenientPlanScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_plan_score("code-154", "name-880");
        assert_eq!(item.status, LenientPlanScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_lenient_plan_score("code-154", "name-880");
        if item.move_to(LenientPlanScoreStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPlanScoreStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_plan_score("code-154", "name-880");
        item.status = LenientPlanScoreStatus::Expired;
        assert!(
            item.move_to(LenientPlanScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
