use super::*;

use acme_kit::text;

/// Where a capped redaction rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRedactionRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedRedactionRuleStatus {
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

impl std::fmt::Display for CappedRedactionRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped redaction rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionRule {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub updated_at: i64,
    pub fingerprint: String,
    pub phone: String,
    pub checksum: String,
    pub cluster: String,
    pub status: CappedRedactionRuleStatus,
}

impl CappedRedactionRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRedactionRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRedactionRuleStatus) -> bool {
        match self.status {
            CappedRedactionRuleStatus::Draft => {
                match next {
                    CappedRedactionRuleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRedactionRuleStatus::Pending => {
                match next {
                    CappedRedactionRuleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRedactionRuleStatus::Active => {
                return next == CappedRedactionRuleStatus::Closed;
            }
            CappedRedactionRuleStatus::Suspended => {
                match next {
                    CappedRedactionRuleStatus::Closed => {
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

    pub fn move_to(&mut self, next: CappedRedactionRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped redaction rule {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "capped redaction rule", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.updated_at as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRedactionRule {
        CappedRedactionRule {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            updated_at: self.updated_at,
            fingerprint: self.fingerprint.clone(),
            phone: self.phone.clone(),
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a capped redaction rule in status draft.
pub fn new_capped_redaction_rule(code: &str, title: &str) -> CappedRedactionRule {
    CappedRedactionRule {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        updated_at: 0,
        fingerprint: "".to_string(),
        phone: "".to_string(),
        checksum: "".to_string(),
        cluster: "".to_string(),
        status: CappedRedactionRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_redaction_rule("code-20", "title-671");
        assert_eq!(item.status, CappedRedactionRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_capped_redaction_rule("code-20", "title-671");
        if item.move_to(CappedRedactionRuleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedRedactionRuleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_redaction_rule("code-20", "title-671");
        item.status = CappedRedactionRuleStatus::Closed;
        assert!(
            item.move_to(CappedRedactionRuleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
