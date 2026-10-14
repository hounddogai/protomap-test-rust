use super::*;

use acme_kit::text;

/// Where a lenient rule request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRuleRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientRuleRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LenientRuleRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient rule request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleRequest {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub attempts: i64,
    pub tags: Vec<String>,
    pub owner_id: String,
    pub approved: bool,
    pub status: LenientRuleRequestStatus,
}

impl LenientRuleRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRuleRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRuleRequestStatus) -> bool {
        match self.status {
            LenientRuleRequestStatus::Draft => {
                return next == LenientRuleRequestStatus::Active;
            }
            LenientRuleRequestStatus::Pending => {
                return next == LenientRuleRequestStatus::Suspended || next == LenientRuleRequestStatus::Closed;
            }
            LenientRuleRequestStatus::Active => {
                match next {
                    LenientRuleRequestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRuleRequestStatus::Suspended => {
                match next {
                    LenientRuleRequestStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientRuleRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient rule request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_total(&self) -> f64 {
        ((self.attempts as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRuleRequest {
        LenientRuleRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            attempts: self.attempts,
            tags: self.tags.clone(),
            owner_id: self.owner_id.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient rule request in status draft.
pub fn new_lenient_rule_request(code: &str, name: &str) -> LenientRuleRequest {
    LenientRuleRequest {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        attempts: 0,
        tags: Vec::new(),
        owner_id: "".to_string(),
        approved: false,
        status: LenientRuleRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_rule_request("code-246", "name-717");
        assert_eq!(item.status, LenientRuleRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_lenient_rule_request("code-246", "name-717");
        if item.move_to(LenientRuleRequestStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRuleRequestStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_rule_request("code-246", "name-717");
        item.status = LenientRuleRequestStatus::Closed;
        assert!(
            item.move_to(LenientRuleRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
