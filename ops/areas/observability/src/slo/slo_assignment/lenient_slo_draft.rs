use super::*;

use acme_kit::text;

/// Where a lenient slo draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSloDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientSloDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientSloDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient slo draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloDraft {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub amount: f64,
    pub version: i64,
    pub pinned: bool,
    pub phone: String,
    pub country: String,
    pub score: f64,
    pub budget: f64,
    pub status: LenientSloDraftStatus,
}

impl LenientSloDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSloDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSloDraftStatus) -> bool {
        match self.status {
            LenientSloDraftStatus::Created => {
                match next {
                    LenientSloDraftStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSloDraftStatus::Validated => {
                return next == LenientSloDraftStatus::Published || next == LenientSloDraftStatus::Retired;
            }
            LenientSloDraftStatus::Published => {
                match next {
                    LenientSloDraftStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientSloDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient slo draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.score - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSloDraft {
        LenientSloDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            amount: self.amount,
            version: self.version,
            pinned: self.pinned,
            phone: self.phone.clone(),
            country: self.country.clone(),
            score: self.score,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a lenient slo draft in status created.
pub fn new_lenient_slo_draft(code: &str, label: &str) -> LenientSloDraft {
    LenientSloDraft {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        amount: 0.0,
        version: 0,
        pinned: false,
        phone: "".to_string(),
        country: "".to_string(),
        score: 0.0,
        budget: 0.0,
        status: LenientSloDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_slo_draft("code-612", "label-836");
        assert_eq!(item.status, LenientSloDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_slo_draft("code-612", "label-836");
        if item.move_to(LenientSloDraftStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSloDraftStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_slo_draft("code-612", "label-836");
        item.status = LenientSloDraftStatus::Retired;
        assert!(
            item.move_to(LenientSloDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
