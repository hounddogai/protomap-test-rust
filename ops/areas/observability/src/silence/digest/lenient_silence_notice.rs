use super::*;

use acme_kit::text;

/// Where a lenient silence notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSilenceNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientSilenceNoticeStatus {
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

impl std::fmt::Display for LenientSilenceNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient silence notice as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceNotice {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub visible: bool,
    pub external_ref: String,
    pub amount: f64,
    pub phone: String,
    pub revision: i64,
    pub principal: String,
    pub status: LenientSilenceNoticeStatus,
}

impl LenientSilenceNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSilenceNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSilenceNoticeStatus) -> bool {
        match self.status {
            LenientSilenceNoticeStatus::New => {
                return next == LenientSilenceNoticeStatus::Queued || next == LenientSilenceNoticeStatus::Running;
            }
            LenientSilenceNoticeStatus::Queued => {
                return next == LenientSilenceNoticeStatus::Running || next == LenientSilenceNoticeStatus::Failed;
            }
            LenientSilenceNoticeStatus::Running => {
                return next == LenientSilenceNoticeStatus::Done;
            }
            LenientSilenceNoticeStatus::Done => {
                return next == LenientSilenceNoticeStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientSilenceNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient silence notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.amount - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSilenceNotice {
        LenientSilenceNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            visible: self.visible,
            external_ref: self.external_ref.clone(),
            amount: self.amount,
            phone: self.phone.clone(),
            revision: self.revision,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient silence notice in status new.
pub fn new_lenient_silence_notice(code: &str, title: &str) -> LenientSilenceNotice {
    LenientSilenceNotice {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        visible: false,
        external_ref: "".to_string(),
        amount: 0.0,
        phone: "".to_string(),
        revision: 0,
        principal: "".to_string(),
        status: LenientSilenceNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_silence_notice("code-772", "title-919");
        assert_eq!(item.status, LenientSilenceNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_lenient_silence_notice("code-772", "title-919");
        if item.move_to(LenientSilenceNoticeStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSilenceNoticeStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_silence_notice("code-772", "title-919");
        item.status = LenientSilenceNoticeStatus::Failed;
        assert!(
            item.move_to(LenientSilenceNoticeStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
