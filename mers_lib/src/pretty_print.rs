use std::{io::Write, process::exit, sync::Arc};

use crate::{
    errors::CheckError,
    prelude_compile::{parse, Source},
    theme::ThemeGen,
};

#[cfg(feature = "ecolor-term")]
pub fn pretty_print(src: Source) {
    if let Err(e) = pretty_print_to(src, &mut std::io::stdout(), DefaultTheme) {
        eprintln!("{e:?}");
        exit(28);
    }
}

/// to print to stdout, use `pretty_print` (available only with the `ecolor-term` feature)
pub fn pretty_print_to<O: Write>(
    mut src: Source,
    out: &mut O,
    theme: impl FTheme<O>,
) -> Result<(), CheckError> {
    let srca = Arc::new(src.clone());
    let parsed = parse(&mut src, &srca)?;
    print_parsed(&srca, parsed.as_ref(), out, theme);
}

pub enum AbstractColor {
    Gray,
    Green,
    Red,
    Blue,
    Cyan,
}
pub fn map_color(color: FColor) -> Option<AbstractColor> {
    Some(match color {
        FColor::Comment => AbstractColor::Gray,
        FColor::Variable => AbstractColor::Green,
        FColor::VariableRef => AbstractColor::Green,
        FColor::If => AbstractColor::Red,
        FColor::IfWithElse => AbstractColor::Red,
        FColor::Loop => AbstractColor::Red,
        FColor::Tuple => AbstractColor::Blue,
        FColor::Object => AbstractColor::Blue,
        FColor::Value => AbstractColor::Cyan,
        FColor::AsType => AbstractColor::Gray,
        FColor::CustomType => AbstractColor::Gray,
        FColor::Unknown => return None,
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FColor {
    Comment,
    Variable,
    VariableRef,
    If,
    IfWithElse,
    Loop,
    Tuple,
    Object,
    Value,
    AsType,
    CustomType,
    Unknown,
}

#[cfg(feature = "ecolor-term")]
pub struct DefaultTheme;
#[cfg(feature = "ecolor-term")]
impl ThemeGen for DefaultTheme {
    type C = FColor;
    type T = std::io::Stdout;
    fn color(&self, text: &str, color: Self::C, t: &mut Self::T) {
        use colored::{Color, Colorize};
        if let Some(color) = map_color(color) {
            let _ = write!(
                t,
                "{}",
                text.color(match color {
                    AbstractColor::Gray => Color::BrightBlack,
                    AbstractColor::Green => Color::Green,
                    AbstractColor::Red => Color::Red,
                    AbstractColor::Blue => Color::Blue,
                    AbstractColor::Cyan => Color::Cyan,
                })
            );
        } else {
            let _ = write!(t, "{}", text);
        }
    }
    fn nocolor(&self, text: &str, t: &mut Self::T) {
        let _ = write!(t, "{}", text);
    }
}

// const FColor::Comment: Color = Color::BrightBlack;
// const FColor::Variable: Color = Color::Green;
// const FColor::Variable_Ref: Color = Color::Green;
// const FColor::If: Color = Color::Red;
// const FColor::If_With_Else: Color = Color::Red;
// const FColor::Loop: Color = Color::Red;
// const FColor::Tuple: Color = Color::Blue;
// const FColor::Object: Color = Color::Blue;
// const FColor::Value: Color = Color::Cyan;
// const FColor::As_Type: Color = Color::BrightBlack;
// const FColor::Custom_Type: Color = Color::BrightBlack;
// const FColor::Unknown: Color = Color::White;

pub trait FTheme<O>: ThemeGen<C = FColor, T = O> {}
impl<O, T: ThemeGen<C = FColor, T = O>> FTheme<O> for T {}

fn print_parsed<O: Write>(
    srca: &Arc<Source>,
    parsed: &dyn crate::program::parsed::MersStatement,
    out: &mut O,
    theme: impl FTheme<O>,
) {
    let mut sections = vec![(FColor::Unknown, srca.src_og().len())];
    build_print(&mut sections, srca, parsed);
    for (start, comment) in srca.comments() {
        let end = start + comment.len();
        build_print_insert_color(&mut sections, FColor::Comment, *start, end);
    }
    let src = srca.src_og();
    let mut i = 0;
    for (clr, end) in sections {
        theme.color(&src[i..end], clr, out);
        i = end;
    }
    let _ = writeln!(out);
}
fn build_print(
    sections: &mut Vec<(FColor, usize)>,
    srca: &Arc<Source>,
    parsed: &dyn crate::program::parsed::MersStatement,
) {
    let any = parsed.as_any();
    let clr = if let Some(v) = any.downcast_ref::<crate::program::parsed::variable::Variable>() {
        if v.is_ref {
            FColor::VariableRef
        } else {
            FColor::Variable
        }
    } else if let Some(v) = any.downcast_ref::<crate::program::parsed::r#if::If>() {
        if v.on_false.is_some() {
            FColor::IfWithElse
        } else {
            FColor::If
        }
    } else if let Some(_) = any.downcast_ref::<crate::program::parsed::r#loop::Loop>() {
        FColor::Loop
    } else if let Some(_) = any.downcast_ref::<crate::program::parsed::tuple::Tuple>() {
        FColor::Tuple
    } else if let Some(_) = any.downcast_ref::<crate::program::parsed::object::Object>() {
        FColor::Object
    } else if let Some(_) = any.downcast_ref::<crate::program::parsed::value::Value>() {
        FColor::Value
    } else if let Some(_) = any.downcast_ref::<crate::program::parsed::as_type::AsType>() {
        FColor::AsType
    } else if let Some(_) = any.downcast_ref::<crate::program::parsed::custom_type::CustomType>() {
        FColor::CustomType
    } else {
        FColor::Unknown
    };
    let range = parsed.source_range();
    let start = srca.pos_in_og(range.start().pos(), true);
    let end = srca.pos_in_og(range.end().pos(), false);
    build_print_insert_color(sections, clr, start, end);
    for s in parsed.inner_statements() {
        build_print(sections, srca, s);
    }
}

fn build_print_insert_color(
    sections: &mut Vec<(FColor, usize)>,
    clr: FColor,
    start: usize,
    end: usize,
) {
    let thing_at_my_start = sections.iter().position(|v| v.1 > start);
    let thing_at_my_end = sections.iter().position(|v| v.1 > end);
    if let Some(thing_at_my_start) = thing_at_my_start {
        if let Some(thing_at_my_end) = thing_at_my_end {
            if thing_at_my_start == thing_at_my_end {
                let (around_color, around_end) = sections[thing_at_my_start];
                if around_color != clr {
                    sections[thing_at_my_start].1 = start;
                    sections.insert(thing_at_my_start + 1, (clr, end));
                    sections.insert(thing_at_my_start + 2, (around_color, around_end));
                    if sections[thing_at_my_start].1
                        == thing_at_my_start
                            .checked_sub(1)
                            .map(|v| sections[v].1)
                            .unwrap_or(0)
                    {
                        // thing at my start now ends at the same place the thing before it ends, so we can remove it
                        sections.remove(thing_at_my_start);
                    }
                }
            } else {
                sections[thing_at_my_start].1 = start;
                sections.insert(thing_at_my_start + 1, (clr, end));
                for _ in 0..(thing_at_my_end.saturating_sub(thing_at_my_start + 1)) {
                    sections.remove(thing_at_my_start + 2);
                }
            }
        } else {
            if sections[thing_at_my_start].0 == clr {
                sections[thing_at_my_start].1 = end;
            } else {
                sections[thing_at_my_start].1 = start;
                sections.push((clr, end));
            }
        }
    } else {
        if let Some(last) = sections.last_mut().filter(|v| v.0 == clr) {
            last.1 = end;
        } else {
            sections.push((clr, end));
        }
    }
}
