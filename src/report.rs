//! Render structured findings without coupling presentation to failure policy.

use crate::compare::Finding;
use crate::finding::Severity;
use crate::model::{ApiModel, Diagnostic};
use crate::policy::FailOn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
    Markdown,
}
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: u8,
    pub tool: Tool,
    pub baseline: Source,
    pub candidate: Source,
    pub summary: Summary,
    pub result: &'static str,
    pub policy_violated: bool,
    pub changes: Vec<Finding>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Serialize)]
pub struct Tool {
    pub name: &'static str,
    pub version: &'static str,
}
#[derive(Debug, Serialize)]
pub struct Source {
    pub path: PathBuf,
}
#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub breaking: usize,
    pub review: usize,
    pub compatible: usize,
    pub informational: usize,
}

impl Report {
    pub fn new(
        baseline: &ApiModel,
        candidate: &ApiModel,
        changes: Vec<Finding>,
        policy: FailOn,
    ) -> Self {
        let mut summary = Summary::default();
        for change in &changes {
            match change.severity {
                Severity::Breaking => summary.breaking += 1,
                Severity::Review => summary.review += 1,
                Severity::Compatible => summary.compatible += 1,
                Severity::Informational => summary.informational += 1,
            }
        }
        let policy_violated = policy.is_violated_by(changes.iter().map(|f| f.severity));
        let result = if summary.breaking > 0 {
            "incompatible"
        } else if summary.review > 0 {
            "review"
        } else {
            "compatible"
        };
        let mut diagnostics = baseline.diagnostics.clone();
        diagnostics.extend(candidate.diagnostics.clone());
        diagnostics.sort_by(|a, b| {
            (&a.source, &a.location, &a.feature).cmp(&(&b.source, &b.location, &b.feature))
        });
        diagnostics.dedup();
        Self {
            schema_version: 1,
            tool: Tool {
                name: "snapapi",
                version: env!("CARGO_PKG_VERSION"),
            },
            baseline: Source {
                path: baseline.source.clone(),
            },
            candidate: Source {
                path: candidate.source.clone(),
            },
            summary,
            result,
            policy_violated,
            changes,
            diagnostics,
        }
    }

    pub fn render(
        &self,
        format: Format,
        include_informational: bool,
    ) -> Result<String, serde_json::Error> {
        if format == Format::Json {
            return serde_json::to_string_pretty(self).map(|s| s + "\n");
        }
        let markdown = format == Format::Markdown;
        let mut output = if markdown {
            "# SnapAPI comparison\n\n".into()
        } else {
            String::new()
        };
        output.push_str(&format!(
            "{} breaking, {} review, {} compatible, {} informational\n\n",
            self.summary.breaking,
            self.summary.review,
            self.summary.compatible,
            self.summary.informational
        ));
        for change in &self.changes {
            if change.severity == Severity::Informational && !include_informational {
                continue;
            }
            if markdown {
                output.push_str(&format!(
                    "## {}: {}\n\n",
                    markdown_escape(change.severity.as_str()),
                    markdown_escape(&change.summary)
                ));
            } else {
                output.push_str(&format!(
                    "{} [{}] {}\n",
                    change.severity.as_str().to_ascii_uppercase(),
                    change.rule_id,
                    change.summary
                ));
            }
            output.push_str(&format!(
                "{}\n\n{}\n\n",
                change
                    .affected_operations
                    .iter()
                    .map(|op| if markdown {
                        markdown_escape(op)
                    } else {
                        op.clone()
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                if markdown {
                    markdown_escape(&change.explanation)
                } else {
                    change.explanation.clone()
                }
            ));
            let value = |v: &Option<serde_json::Value>| {
                v.as_ref()
                    .map(serde_json::Value::to_string)
                    .unwrap_or_else(|| "<absent>".into())
            };
            if markdown {
                output.push_str(&format!(
                    "Location: {}\n\nBefore: {}\n\nAfter: {}\n\n[{}]({})\n\n",
                    markdown_escape(&change.location),
                    markdown_escape(&value(&change.before)),
                    markdown_escape(&value(&change.after)),
                    change.rule_id,
                    change.documentation
                ));
            } else {
                output.push_str(&format!(
                    "  {}\n  {} -> {}\n  Rule: {}\n\n",
                    change.location,
                    value(&change.before),
                    value(&change.after),
                    change.documentation
                ));
            }
        }
        for diagnostic in &self.diagnostics {
            let message = format!(
                "Unsupported {} at {}:{} — {}",
                diagnostic.feature,
                diagnostic.source.display(),
                diagnostic.location,
                diagnostic.explanation
            );
            output.push_str(&format!(
                "{}\n",
                if markdown {
                    markdown_escape(&message)
                } else {
                    message
                }
            ));
        }
        Ok(output)
    }
}

fn markdown_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('\n', " ")
}
