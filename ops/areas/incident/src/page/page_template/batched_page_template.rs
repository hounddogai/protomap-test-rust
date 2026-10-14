use super::*;

use acme_kit::text;

/// Where a batched page template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPageTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedPageTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedPageTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched page template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageTemplate {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub ratio: f64,
    pub notes: String,
    pub status: BatchedPageTemplateStatus,
}

impl BatchedPageTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPageTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPageTemplateStatus) -> bool {
        match self.status {
            BatchedPageTemplateStatus::Idle => {
                match next {
                    BatchedPageTemplateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPageTemplateStatus::Warming => {
                match next {
                    BatchedPageTemplateStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPageTemplateStatus::Ready => {
                match next {
                    BatchedPageTemplateStatus::Draining | BatchedPageTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPageTemplateStatus::Draining => {
                return next == BatchedPageTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPageTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched page template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> BatchedPageTemplate {
        BatchedPageTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            ratio: self.ratio,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched page template in status idle.
pub fn new_batched_page_template(code: &str, label: &str) -> BatchedPageTemplate {
    BatchedPageTemplate {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        ratio: 0.0,
        notes: "".to_string(),
        status: BatchedPageTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_page_template("code-364", "label-624");
        assert_eq!(item.status, BatchedPageTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_batched_page_template("code-364", "label-624");
        if item.move_to(BatchedPageTemplateStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPageTemplateStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_page_template("code-364", "label-624");
        item.status = BatchedPageTemplateStatus::Stopped;
        assert!(
            item.move_to(BatchedPageTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
