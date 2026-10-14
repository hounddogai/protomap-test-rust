use super::*;

use acme_kit::text;

/// Where a legacy quote group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyQuoteGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyQuoteGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyQuoteGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy quote group as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteGroup {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub score: f64,
    pub revision: i64,
    pub owner_id: String,
    pub issuer: String,
    pub status: LegacyQuoteGroupStatus,
}

impl LegacyQuoteGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyQuoteGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyQuoteGroupStatus) -> bool {
        match self.status {
            LegacyQuoteGroupStatus::Created => {
                return next == LegacyQuoteGroupStatus::Validated || next == LegacyQuoteGroupStatus::Published;
            }
            LegacyQuoteGroupStatus::Validated => {
                match next {
                    LegacyQuoteGroupStatus::Published | LegacyQuoteGroupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyQuoteGroupStatus::Published => {
                match next {
                    LegacyQuoteGroupStatus::Retired => {
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

    pub fn move_to(&mut self, next: LegacyQuoteGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy quote group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.score - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyQuoteGroup {
        LegacyQuoteGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            score: self.score,
            revision: self.revision,
            owner_id: self.owner_id.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a legacy quote group in status created.
pub fn new_legacy_quote_group(code: &str, name: &str) -> LegacyQuoteGroup {
    LegacyQuoteGroup {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        score: 0.0,
        revision: 0,
        owner_id: "".to_string(),
        issuer: "".to_string(),
        status: LegacyQuoteGroupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_quote_group("code-211", "name-851");
        assert_eq!(item.status, LegacyQuoteGroupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_legacy_quote_group("code-211", "name-851");
        if item.move_to(LegacyQuoteGroupStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyQuoteGroupStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_quote_group("code-211", "name-851");
        item.status = LegacyQuoteGroupStatus::Retired;
        assert!(
            item.move_to(LegacyQuoteGroupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
