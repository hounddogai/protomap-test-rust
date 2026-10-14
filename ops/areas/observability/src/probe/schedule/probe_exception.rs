use super::*;

use acme_kit::text;

/// Where a probe exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ProbeExceptionStatus {
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

impl std::fmt::Display for ProbeExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe exception as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeException {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub issuer: String,
    pub label: String,
    pub status: ProbeExceptionStatus,
}

impl ProbeException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeExceptionStatus) -> bool {
        match self.status {
            ProbeExceptionStatus::New => {
                match next {
                    ProbeExceptionStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeExceptionStatus::Queued => {
                return next == ProbeExceptionStatus::Running;
            }
            ProbeExceptionStatus::Running => {
                match next {
                    ProbeExceptionStatus::Done | ProbeExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeExceptionStatus::Done => {
                return next == ProbeExceptionStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "probe exception", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> ProbeException {
        ProbeException {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            issuer: self.issuer.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a probe exception in status new.
pub fn new_probe_exception(code: &str, title: &str) -> ProbeException {
    ProbeException {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        issuer: "".to_string(),
        label: "".to_string(),
        status: ProbeExceptionStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_probe_exception("code-576", "title-523");
        assert_eq!(item.status, ProbeExceptionStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_probe_exception("code-576", "title-523");
        if item.move_to(ProbeExceptionStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeExceptionStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_probe_exception("code-576", "title-523");
        item.status = ProbeExceptionStatus::Failed;
        assert!(
            item.move_to(ProbeExceptionStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
