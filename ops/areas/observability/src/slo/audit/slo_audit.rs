use super::*;

use acme_kit::text;

/// Where a slo audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloAuditStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SloAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SloAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a slo audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SloAudit {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub threshold: f64,
    pub locale: String,
    pub retries: i64,
    pub archived: bool,
    pub burst: i64,
    pub attempts: i64,
    pub region: String,
    pub status: SloAuditStatus,
}

impl SloAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloAuditStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloAuditStatus) -> bool {
        match self.status {
            SloAuditStatus::Created => {
                match next {
                    SloAuditStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloAuditStatus::Validated => {
                return next == SloAuditStatus::Retired;
            }
            SloAuditStatus::Published => {
                return next == SloAuditStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.retries as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloAudit {
        SloAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            threshold: self.threshold,
            locale: self.locale.clone(),
            retries: self.retries,
            archived: self.archived,
            burst: self.burst,
            attempts: self.attempts,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a slo audit in status created.
pub fn new_slo_audit(code: &str, title: &str) -> SloAudit {
    SloAudit {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        threshold: 0.0,
        locale: "".to_string(),
        retries: 0,
        archived: false,
        burst: 0,
        attempts: 0,
        region: "".to_string(),
        status: SloAuditStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_slo_audit("code-167", "title-89");
        assert_eq!(item.status, SloAuditStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_slo_audit("code-167", "title-89");
        if item.move_to(SloAuditStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloAuditStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_slo_audit("code-167", "title-89");
        item.status = SloAuditStatus::Retired;
        assert!(item.move_to(SloAuditStatus::Created).is_err(), "expected move to fail");
    }
}
