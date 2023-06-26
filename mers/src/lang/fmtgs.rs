use std::fmt::{Display, Formatter};

use super::global_info::{ColorFormatMode, ColorFormatter, GlobalScriptInfo};

use colorize::AnsiColor;

pub enum Color {
    // Keep,
    Grey,
    Red,
    Yellow,
    Green,
    Blue,
    Cyan,
    Magenta,
}
impl Color {
    pub fn colorize(&self, s: String) -> String {
        match self {
            // Self::Keep => s,
            Self::Grey => s.grey().to_string(),
            Self::Red => s.red().to_string(),
            Self::Yellow => s.yellow().to_string(),
            Self::Green => s.green().to_string(),
            Self::Blue => s.blue().to_string(),
            Self::Cyan => s.cyan().to_string(),
            Self::Magenta => s.magenta().to_string(),
        }
    }
}

#[derive(Default)]
pub struct FormatInfo {
    pub depth: usize,
    pub brackets: usize,
}
impl FormatInfo {
    fn color<F>(&self, info: Option<&GlobalScriptInfo>, color: F, s: String) -> String
    where
        F: Fn(&ColorFormatter) -> &Color,
    {
        if let Some(info) = info {
            let color = color(&info.formatter);
            match info.formatter.mode {
                ColorFormatMode::Plain => s,
                ColorFormatMode::Colorize => color.colorize(s),
            }
        } else {
            s
        }
    }
    pub fn open_bracket(&mut self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        let o = self.color(
            info,
            |c| &c.bracket_colors[self.brackets % c.bracket_colors.len()],
            s,
        );
        self.brackets += 1;
        o
    }
    pub fn close_bracket(&mut self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.brackets -= 1;
        self.color(
            info,
            |c| &c.bracket_colors[self.brackets % c.bracket_colors.len()],
            s,
        )
    }
    pub fn go_deeper(&mut self) {
        self.depth += 1;
    }
    pub fn go_shallower(&mut self) {
        self.depth -= 1;
    }
    pub fn variable_ref_symbol(&self, _info: Option<&GlobalScriptInfo>, s: String) -> String {
        s
    }
    pub fn variable(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.variable_color, s)
    }
    pub fn if_if(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.keyword_if_color, s)
    }
    pub fn if_else(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.keyword_else_color, s)
    }
    pub fn loop_loop(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.keyword_loop_color, s)
    }
    pub fn loop_for(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.keyword_for_color, s)
    }
    pub fn kw_switch(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.keyword_switch_color, s)
    }
    pub fn kw_match(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.keyword_match_color, s)
    }
    pub fn fncall(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.function_call_color, s)
    }
    pub fn fndef_fn(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.function_def_fn_color, s)
    }
    pub fn fndef_name(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.function_def_name_color, s)
    }
    pub fn value_string_quotes(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.value_string_quotes_color, s)
    }
    pub fn value_string_content(&self, info: Option<&GlobalScriptInfo>, s: String) -> String {
        self.color(info, |c| &c.value_string_content_color, s)
    }
    pub fn line_prefix(&self) -> String {
        "    ".repeat(self.depth)
    }
}

pub trait FormatGs {
    fn fmtgs(
        &self,
        f: &mut Formatter,
        info: Option<&GlobalScriptInfo>,
        form: &mut FormatInfo,
        file: Option<&crate::parsing::file::File>,
    ) -> std::fmt::Result;
    fn with_info<'a>(&'a self, info: &'a GlobalScriptInfo) -> FormatWithGs<'a, Self> {
        FormatWithGs {
            format: &self,
            info: Some(info),
            file: None,
        }
    }
    fn with_file<'a>(&'a self, file: &'a crate::parsing::file::File) -> FormatWithGs<'a, Self> {
        FormatWithGs {
            format: &self,
            info: None,
            file: Some(file),
        }
    }
    fn with_info_and_file<'a>(
        &'a self,
        info: &'a GlobalScriptInfo,
        file: &'a crate::parsing::file::File,
    ) -> FormatWithGs<'a, Self> {
        FormatWithGs {
            format: &self,
            info: Some(info),
            file: Some(file),
        }
    }
    fn with<'a>(
        &'a self,
        info: Option<&'a GlobalScriptInfo>,
        file: Option<&'a crate::parsing::file::File>,
    ) -> FormatWithGs<'a, Self> {
        FormatWithGs {
            format: &self,
            info,
            file,
        }
    }
}
pub struct FormatWithGs<'a, T: ?Sized>
where
    T: FormatGs,
{
    format: &'a T,
    info: Option<&'a GlobalScriptInfo>,
    file: Option<&'a crate::parsing::file::File>,
}
impl<'a, T> Display for FormatWithGs<'a, T>
where
    T: FormatGs,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format
            .fmtgs(f, self.info, &mut FormatInfo::default(), self.file.clone())
    }
}
