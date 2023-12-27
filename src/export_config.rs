use std::{borrow::Cow, path::PathBuf};

use specta::ts::FormatterFn;

/// ExportConfig is used to configure how rspc will export your types.
pub struct ExportConfig {
    pub(crate) export_path: PathBuf,
    pub(crate) header: Cow<'static, str>,
    pub(crate) formatter: Option<FormatterFn>,
}

impl ExportConfig {
    pub fn new(export_path: impl Into<PathBuf>) -> ExportConfig {
        ExportConfig {
            export_path: export_path.into(),
            header: Cow::Borrowed(""),
            formatter: None,
        }
    }

    pub fn header(self, header: impl Into<Cow<'static, str>>) -> Self {
        Self {
            header: header.into(),
            ..self
        }
    }

    pub fn formatter(self, formatter: FormatterFn) -> Self {
        Self {
            formatter: Some(formatter),
            ..self
        }
    }
}
