use super::{EColor, ThemeGen};

pub struct NoTheme;
impl ThemeGen for NoTheme {
    type C = EColor;
    type T = String;
    fn color(&self, text: &str, _color: EColor, t: &mut String) {
        self.nocolor(text, t);
    }
    fn nocolor(&self, text: &str, t: &mut String) {
        t.push_str(text);
    }
}

/// converts an `EColor` to the color type you need for your theme.
/// This theme is optimized for ANSI terminal colors,
/// as most of mers' colored output will be printed to a terminal.
pub fn default_theme<C>(
    color: EColor,
    gray: C,
    red: C,
    red_bright: C,
    green: C,
    green_bright: C,
    yellow: C,
    yellow_bright: C,
    blue: C,
    blue_bright: C,
    magenta: C,
    magenta_bright: C,
    cyan: C,
    cyan_bright: C,
) -> Option<C> {
    if let Indent(n) = color {
        return Some(match n % 7 {
            1 => red,
            2 => green,
            3 => yellow,
            4 => blue,
            5 => magenta,
            6 => cyan,
            _ => return None,
        });
    }
    let hard_err = red;
    let type_right = blue;
    let type_wrong = magenta;
    let type_wrong_b = magenta_bright;
    let function = blue_bright; // used in combination with TYPE_WRONG
    let missing = cyan;
    let runtime = yellow;
    let runtime_b = yellow_bright;
    let unused = green;
    let unused_b = green_bright;
    drop(red_bright);
    drop(cyan_bright);
    use EColor::*;
    Some(match color {
        Indent(_) => unreachable!(),

        // macros (#...)
        WhitespaceAfterHashtag
        | HashUnknown
        | HashIncludeCantLoadFile
        | HashIncludeNotAString
        | HashIncludeErrorInIncludedFile
        | StacktraceDescendHashInclude => hard_err,

        // -- bad syntax --
        UnknownVariable => hard_err,
        BackslashEscapeUnknown => hard_err,
        BackslashEscapeEOF | StringEOF | TypeEOF => missing,
        BadCharInTupleType => hard_err,
        BadCharInFunctionType => hard_err,
        TryBadSyntax => hard_err,
        TypeAnnotationNoClosingBracket | BracketedRefTypeNoClosingBracket => missing,

        BadTypeFromParsed => type_wrong_b,

        // -- type-errors --
        IfConditionNotBool => type_wrong,
        TryNoFunctionFound => type_wrong_b,
        TryNotAFunction => type_wrong,
        TryUnusedFunction1 => unused,
        TryUnusedFunction2 => unused_b,

        CustomTypeTestFailed => hard_err,

        ChainWithNonFunction => type_wrong,

        AssignTargetNonReference => type_wrong,

        Function => function,
        FunctionArgument => type_wrong,

        InitFrom | AssignFrom | AsTypeStatementWithTooBroadType => type_wrong,
        InitTo | AssignTo | AsTypeTypeAnnotation => type_right,

        // -- runtime-errors --
        StacktraceDescend => runtime,
        MaximumRuntimeExceeded => runtime_b,

        InCodePositionLine => gray,
    })
}

#[cfg(feature = "ecolor-term")]
pub struct TermDefaultTheme;
#[cfg(feature = "ecolor-term")]
impl ThemeGen for TermDefaultTheme {
    type C = EColor;
    type T = String;
    fn color(&self, text: &str, color: EColor, t: &mut String) {
        use colored::Color::*;
        if let Some(color) = default_theme(
            color,
            BrightBlack,
            Red,
            BrightRed,
            Green,
            BrightGreen,
            Yellow,
            BrightYellow,
            Blue,
            BrightBlue,
            Magenta,
            BrightMagenta,
            Cyan,
            BrightCyan,
        ) {
            use colored::Colorize;
            t.push_str(&text.color(color).to_string());
        } else {
            self.nocolor(text, t)
        }
    }
    fn nocolor(&self, text: &str, t: &mut String) {
        t.push_str(text);
    }
}

#[cfg(feature = "ecolor-html")]
pub struct HtmlDefaultTheme;
#[cfg(feature = "ecolor-html")]
impl ThemeGen for HtmlDefaultTheme {
    type C = EColor;
    type T = String;
    fn color(&self, text: &str, color: EColor, t: &mut String) {
        if let Some(color) = default_theme(
            color,
            "Gray",
            "Crimson",
            "Red",
            "Green",
            "LimeGreen",
            "Gold",
            "Yellow",
            "RoyalBlue",
            "DeepSkyBlue",
            "Purple",
            "Orchid",
            "DarkCyan",
            "Turquoise",
        ) {
            t.push_str("<span style=\"color:");
            t.push_str(color);
            t.push_str(";\">");
            self.nocolor(text, t);
            t.push_str("</span>");
        } else {
            self.nocolor(text, t);
        }
    }
    fn nocolor(&self, text: &str, t: &mut String) {
        t.push_str(html_escape::encode_text(text).replace("\n", "<br>\n"));
    }
}
