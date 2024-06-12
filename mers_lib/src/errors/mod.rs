use std::{
    fmt::{Debug, Display},
    rc::Rc,
    sync::{atomic::AtomicUsize, Arc},
};

use colored::{ColoredString, Colorize};
use line_span::LineSpans;

#[cfg(feature = "parse")]
use crate::parsing::Source;

#[derive(Clone, Copy, Debug)]
pub struct SourcePos(pub(crate) usize);
impl SourcePos {
    pub fn pos(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct SourceRange {
    in_file: Arc<Source>,
    start: SourcePos,
    end: SourcePos,
}
impl From<(SourcePos, SourcePos, &Arc<Source>)> for SourceRange {
    fn from(value: (SourcePos, SourcePos, &Arc<Source>)) -> Self {
        SourceRange {
            in_file: Arc::clone(value.2),
            start: value.0,
            end: value.1,
        }
    }
}
impl SourceRange {
    pub fn start(&self) -> SourcePos {
        self.start
    }
    pub fn end(&self) -> SourcePos {
        self.end
    }
    pub fn in_file(&self) -> &Arc<Source> {
        &self.in_file
    }
}
#[derive(Clone)]
pub struct CheckError(pub Vec<CheckErrorComponent>);
#[allow(non_upper_case_globals)]
pub mod error_colors {
    use colored::Color;

    pub const UnknownVariable: Color = Color::Red;

    pub const WhitespaceAfterHashtag: Color = Color::Red;
    pub const HashUnknown: Color = Color::Red;
    pub const HashIncludeCantLoadFile: Color = Color::Red;
    pub const HashIncludeNotAString: Color = Color::Red;
    pub const HashIncludeErrorInIncludedFile: Color = Color::Red;

    pub const BackslashEscapeUnknown: Color = Color::Red;
    pub const BackslashEscapeEOF: Color = Color::Red;
    pub const StringEOF: Color = Color::Red;

    pub const IfConditionNotBool: Color = Color::Red;
    pub const ChainWithNonFunction: Color = Color::Yellow;

    pub const Function: Color = Color::BrightMagenta;
    pub const FunctionArgument: Color = Color::BrightBlue;

    pub const InitFrom: Color = Color::BrightCyan;
    pub const InitTo: Color = Color::Green;
    pub const AssignFrom: Color = InitFrom;
    pub const AssignTo: Color = InitTo;
    pub const AssignTargetNonReference: Color = Color::BrightYellow;

    pub const AsTypeStatementWithTooBroadType: Color = InitFrom;
    pub const AsTypeTypeAnnotation: Color = InitTo;

    pub const BadCharInTupleType: Color = Color::Red;
    pub const BadCharInFunctionType: Color = Color::Red;
    pub const BadTypeFromParsed: Color = Color::Blue;
    pub const TypeAnnotationNoClosingBracket: Color = Color::Blue;

    pub const TryBadSyntax: Color = Color::Red;
    pub const TryNoFunctionFound: Color = Color::Red;
    pub const TryNotAFunction: Color = Color::Red;
    pub const TryUnusedFunction1: Color = Color::Red;
    pub const TryUnusedFunction2: Color = Color::BrightRed;
}
#[derive(Clone)]
pub enum CheckErrorComponent {
    Message(String),
    Error(CheckError),
    ErrorWithDifferentSource(CheckError),
    Source(Vec<(SourceRange, Option<colored::Color>)>),
}
#[derive(Clone)]
pub struct CheckErrorHRConfig {
    color_index: Rc<AtomicUsize>,
    indent_start: String,
    indent_default: String,
    indent_end: String,
    /// if true, shows "original" source code, if false, shows source with comments removed (this is what the parser uses internally)
    show_comments: bool,
}
#[cfg(feature = "parse")]
pub struct CheckErrorDisplay<'a> {
    e: &'a CheckError,
    pub show_comments: bool,
}
#[cfg(feature = "parse")]
impl<'a> CheckErrorDisplay<'a> {
    pub fn show_comments(mut self, show_comments: bool) -> Self {
        self.show_comments = show_comments;
        self
    }
}
#[cfg(feature = "parse")]
impl Display for CheckErrorDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.e.human_readable(
            f,
            &CheckErrorHRConfig {
                color_index: Rc::new(AtomicUsize::new(0)),
                indent_start: String::new(),
                indent_default: String::new(),
                indent_end: String::new(),
                show_comments: self.show_comments,
            },
        )
    }
}
#[allow(unused)]
impl CheckError {
    pub fn new() -> Self {
        CheckError(vec![])
    }
    fn add(mut self, v: CheckErrorComponent) -> Self {
        self.0.push(v);
        self
    }
    fn add_mut(&mut self, v: CheckErrorComponent) -> &mut Self {
        self.0.push(v);
        self
    }
    pub(crate) fn msg(self, s: String) -> Self {
        self.add(CheckErrorComponent::Message(s))
    }
    pub(crate) fn msg_mut(&mut self, s: String) -> &mut Self {
        self.add_mut(CheckErrorComponent::Message(s))
    }
    pub(crate) fn err(self, e: Self) -> Self {
        self.add(CheckErrorComponent::Error(e))
    }
    pub(crate) fn err_mut(&mut self, e: Self) -> &mut Self {
        self.add_mut(CheckErrorComponent::Error(e))
    }
    pub(crate) fn err_with_diff_src(self, e: CheckError) -> Self {
        self.add(CheckErrorComponent::ErrorWithDifferentSource(e))
    }
    pub(crate) fn err_with_diff_src_mut(&mut self, e: CheckError) -> &mut Self {
        self.add_mut(CheckErrorComponent::ErrorWithDifferentSource(e))
    }
    pub(crate) fn src(self, s: Vec<(SourceRange, Option<colored::Color>)>) -> Self {
        self.add(CheckErrorComponent::Source(s))
    }
    pub(crate) fn src_mut(&mut self, s: Vec<(SourceRange, Option<colored::Color>)>) -> &mut Self {
        self.add_mut(CheckErrorComponent::Source(s))
    }
    #[cfg(feature = "parse")]
    pub fn display<'a>(&'a self) -> CheckErrorDisplay<'a> {
        CheckErrorDisplay {
            e: self,
            show_comments: true,
        }
    }
    /// will, unless empty, end in a newline
    #[cfg(feature = "parse")]
    fn human_readable(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        cfg: &CheckErrorHRConfig,
    ) -> std::fmt::Result {
        use crate::parsing::SourceFrom;

        let len = self.0.len();
        for (i, component) in self.0.iter().enumerate() {
            macro_rules! indent {
                ($s:expr, $e:expr) => {
                    if $e && i + 1 == len {
                        &cfg.indent_end
                    } else if $s && i == 0 {
                        &cfg.indent_start
                    } else {
                        &cfg.indent_default
                    }
                };
            }
            match component {
                CheckErrorComponent::Message(msg) => {
                    let lines = msg.lines().collect::<Vec<_>>();
                    let lc = lines.len();
                    for (i, line) in lines.into_iter().enumerate() {
                        writeln!(f, "{}{line}", indent!(i == 0, i + 1 == lc))?
                    }
                }
                CheckErrorComponent::Error(err) => {
                    let clr = Self::get_color(&cfg.color_index);
                    let mut cfg = cfg.clone();
                    cfg.indent_start.push_str(&clr("│").to_string());
                    cfg.indent_default.push_str(&clr("│").to_string());
                    cfg.indent_end.push_str(&clr("└").to_string());
                    err.human_readable(f, &cfg)?;
                }
                CheckErrorComponent::ErrorWithDifferentSource(err) => {
                    let clr = Self::get_color(&cfg.color_index);
                    let mut cfg = cfg.clone();
                    cfg.indent_start.push_str(&clr("┃").bold().to_string());
                    cfg.indent_default.push_str(&clr("┃").bold().to_string());
                    cfg.indent_end.push_str(&clr("┗").bold().to_string());
                    err.human_readable(f, &cfg)?;
                }
                CheckErrorComponent::Source(highlights) => {
                    if let Some(src) = highlights.first().map(|v| v.0.in_file.as_ref()) {
                        let start = highlights.iter().map(|v| v.0.start.pos()).min();
                        let end = highlights.iter().map(|v| v.0.end.pos()).max();
                        if let (Some(start_in_line), Some(end_in_line)) = (start, end) {
                            let start = src.get_line_start(start_in_line);
                            let end = src.get_line_end(end_in_line);
                            let (start_with_comments, end_with_comments) = (
                                src.pos_in_og(start_in_line, true),
                                src.pos_in_og(end_in_line, false),
                            );
                            let (mut start, mut end) = if cfg.show_comments {
                                (src.pos_in_og(start, true), src.pos_in_og(end, false))
                            } else {
                                (start, end)
                            };
                            let mut first_line_start = 0;
                            let first_line_nr = src
                                .src_og()
                                .line_spans()
                                .take_while(|l| {
                                    if l.start() <= start_with_comments {
                                        first_line_start = l.start();
                                        true
                                    } else {
                                        false
                                    }
                                })
                                .count();
                            if cfg.show_comments && first_line_start < start {
                                start = first_line_start;
                            }
                            let mut last_line_start = 0;
                            let last_line_nr = src
                                .src_og()
                                .line_spans()
                                .take_while(|l| {
                                    if l.start() <= end_with_comments {
                                        last_line_start = l.start();
                                        if cfg.show_comments && l.end() > end {
                                            end = l.end();
                                        }
                                        true
                                    } else {
                                        false
                                    }
                                })
                                .count();
                            let src_from = match src.src_from() {
                                SourceFrom::File(path) => format!(" [{}]", path.to_string_lossy()),
                                SourceFrom::Unspecified => String::with_capacity(0),
                            };
                            if first_line_nr == last_line_nr {
                                writeln!(
                                    f,
                                    "{}",
                                    format!(
                                        "{}Line {first_line_nr} ({}..{}){}",
                                        indent!(true, false),
                                        start_with_comments + 1 - first_line_start,
                                        end_with_comments - last_line_start,
                                        src_from,
                                    )
                                    .bright_black()
                                )?;
                            } else {
                                writeln!(
                                    f,
                                    "{}",
                                    format!(
                                        "{}Lines {first_line_nr}-{last_line_nr} ({}..{}){}",
                                        indent!(true, false),
                                        start_with_comments + 1 - first_line_start,
                                        end_with_comments - last_line_start,
                                        src_from,
                                    )
                                    .bright_black()
                                )?;
                            }
                            let lines = if cfg.show_comments {
                                src.src_og()[start..end].line_spans().collect::<Vec<_>>()
                            } else {
                                src.src()[start..end].line_spans().collect::<Vec<_>>()
                            };
                            let lines_count = lines.len();
                            for (line_nr_rel, line) in lines.into_iter().enumerate() {
                                let last_line = line_nr_rel + 1 == lines_count;
                                let line_start = line.start();
                                let line_end = line.end();
                                let line = line.as_str();
                                let mut line_printed = false;
                                let mut right = 0;
                                for (pos, color) in highlights {
                                    if let Some(color) = color {
                                        let (highlight_start, highlight_end) = if cfg.show_comments
                                        {
                                            (
                                                src.pos_in_og(pos.start.pos(), true),
                                                src.pos_in_og(pos.end.pos(), false),
                                            )
                                        } else {
                                            (pos.start.pos(), pos.end.pos())
                                        };
                                        let highlight_start = highlight_start - start;
                                        let highlight_end = highlight_end - start;
                                        if highlight_start < line_end && highlight_end > line_start
                                        {
                                            if !line_printed {
                                                // this isn't the last line (important for indent)
                                                writeln!(f, "{} {line}", indent!(false, false))?;
                                                line_printed = true;
                                            }
                                            // where the highlight starts in this line
                                            let hl_start =
                                                highlight_start.saturating_sub(line_start);
                                            // highlight would be further left than cursor, so we need a new line
                                            if hl_start < right {
                                                right = 0;
                                                writeln!(f)?;
                                            }
                                            // length of the highlight
                                            let hl_len = highlight_end
                                                .saturating_sub(line_start)
                                                .saturating_sub(hl_start);
                                            let hl_space = hl_start - right;
                                            let print_indent = right == 0;
                                            let hl_len = hl_len.min(line.len() - right);
                                            right += hl_space + hl_len;
                                            if print_indent && right != 0 {
                                                write!(f, "{} ", indent!(false, false))?;
                                            }
                                            write!(
                                                f,
                                                "{}{}",
                                                " ".repeat(hl_space),
                                                "~".repeat(hl_len).color(*color)
                                            )?;
                                        }
                                    }
                                }
                                if !line_printed {
                                    // may be last line (important for indent)
                                    writeln!(f, "{} {line}", indent!(false, last_line))?;
                                }
                                if right != 0 {
                                    writeln!(f)?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn get_color(i: &AtomicUsize) -> impl Fn(&str) -> ColoredString {
        let i = i.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        move |s| match i % 8 {
            0 => s.bright_white(),
            1 => s.bright_green(),
            2 => s.bright_purple(),
            3 => s.bright_cyan(),
            4 => s.bright_red(),
            5 => s.bright_yellow(),
            6 => s.bright_magenta(),
            _ => s.bright_blue(),
        }
    }
}
impl From<String> for CheckError {
    fn from(value: String) -> Self {
        Self::new().msg(value)
    }
}
impl Debug for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
impl Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}
