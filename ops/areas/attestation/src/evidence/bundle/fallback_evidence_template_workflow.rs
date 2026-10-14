use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves fallback evidence templates through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackEvidenceTemplateService {
    pub store: Rc<dyn FallbackEvidenceTemplateStore>,
    pub validator: FallbackEvidenceTemplateValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl FallbackEvidenceTemplateService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback evidence template {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackEvidenceTemplateStatus::Scheduled)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is scheduled", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback evidence template {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackEvidenceTemplateStatus::Cancelled)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is cancelled: {}", code, reason));
        Ok(())
    }

    pub fn review_stale(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            let problems = self.validator.validate(&item);
            if (problems.len() as i64) == 0 {
                continue;
            }
            self.note(&format!("{} needs attention: {}", item.code, problems.join(", ")));
            changed += 1;
        }
        changed
    }
}

pub fn new_fallback_evidence_template_service(
    store: Rc<dyn FallbackEvidenceTemplateStore>,
    clock: Rc<dyn clock::Clock>,
) -> FallbackEvidenceTemplateService {
    FallbackEvidenceTemplateService {
        store: store.clone(),
        validator: default_fallback_evidence_template_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
