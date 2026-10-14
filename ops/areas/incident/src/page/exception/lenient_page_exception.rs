use super::*;

use acme_kit::text;

/// Where a lenient page exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPageExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientPageExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LenientPageExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient page exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageException {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub starts_at: i64,
    pub title: String,
    pub checksum: String,
    pub retries: i64,
    pub timezone: String,
    pub window_days: i64,
    pub status: LenientPageExceptionStatus,
}

impl LenientPageException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPageExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPageExceptionStatus) -> bool {
        match self.status {
            LenientPageExceptionStatus::Open => {
                return next == LenientPageExceptionStatus::Reopened;
            }
            LenientPageExceptionStatus::InReview => {
                match next {
                    LenientPageExceptionStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPageExceptionStatus::Resolved => {
                match next {
                    LenientPageExceptionStatus::Reopened | LenientPageExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPageExceptionStatus::Reopened => {
                match next {
                    LenientPageExceptionStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientPageExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient page exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.window_days as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPageException {
        LenientPageException {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            starts_at: self.starts_at,
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            retries: self.retries,
            timezone: self.timezone.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient page exception in status open.
pub fn new_lenient_page_exception(code: &str, name: &str) -> LenientPageException {
    LenientPageException {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        starts_at: 0,
        title: "".to_string(),
        checksum: "".to_string(),
        retries: 0,
        timezone: "".to_string(),
        window_days: 0,
        status: LenientPageExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_page_exception("code-28", "name-455");
        assert_eq!(item.status, LenientPageExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_lenient_page_exception("code-28", "name-455");
        if item.move_to(LenientPageExceptionStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPageExceptionStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_page_exception("code-28", "name-455");
        item.status = LenientPageExceptionStatus::Archived;
        assert!(
            item.move_to(LenientPageExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
