use super::*;

use acme_kit::text;

/// Where a regional sink template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSinkTemplateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalSinkTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalSinkTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional sink template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSinkTemplate {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub burst: i64,
    pub sequence: i64,
    pub threshold: f64,
    pub phone: String,
    pub status: RegionalSinkTemplateStatus,
}

impl RegionalSinkTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSinkTemplateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSinkTemplateStatus) -> bool {
        match self.status {
            RegionalSinkTemplateStatus::Requested => {
                return next == RegionalSinkTemplateStatus::Expired;
            }
            RegionalSinkTemplateStatus::Granted => {
                match next {
                    RegionalSinkTemplateStatus::Revoked | RegionalSinkTemplateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSinkTemplateStatus::Revoked => {
                return next == RegionalSinkTemplateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalSinkTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional sink template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.burst as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSinkTemplate {
        RegionalSinkTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            burst: self.burst,
            sequence: self.sequence,
            threshold: self.threshold,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional sink template in status requested.
pub fn new_regional_sink_template(code: &str, label: &str) -> RegionalSinkTemplate {
    RegionalSinkTemplate {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        burst: 0,
        sequence: 0,
        threshold: 0.0,
        phone: "".to_string(),
        status: RegionalSinkTemplateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_sink_template("code-580", "label-348");
        assert_eq!(item.status, RegionalSinkTemplateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_regional_sink_template("code-580", "label-348");
        if item.move_to(RegionalSinkTemplateStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalSinkTemplateStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_sink_template("code-580", "label-348");
        item.status = RegionalSinkTemplateStatus::Expired;
        assert!(
            item.move_to(RegionalSinkTemplateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
