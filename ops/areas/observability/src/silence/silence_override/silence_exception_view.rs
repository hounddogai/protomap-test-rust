use super::*;

use acme_kit::text;

/// How screens and exports show a silence exception.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceExceptionView {
    pub code: String,
    pub label: String,
    pub retries: String,
    pub name: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_silence_exception_view(item: &SilenceException) -> SilenceExceptionView {
    SilenceExceptionView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        retries: item.retries.to_string(),
        name: item.name.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_exception_views(items: &[SilenceException]) -> Vec<SilenceExceptionView> {
    let mut views: Vec<SilenceExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_exception_view(item));
    }
    views
}

pub fn sum_silence_exception_view(view: &SilenceExceptionView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_retries as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_exception("code-328", "label-422");
        let view = to_silence_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
