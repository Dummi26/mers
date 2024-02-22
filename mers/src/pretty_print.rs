use std::{process::exit, sync::Arc};

use colored::{Color, Colorize};
use mers_lib::prelude_compile::{parse, Source};

pub fn pretty_print(mut src: Source) {
    let srca = Arc::new(src.clone());
    match parse(&mut src, &srca) {
        Err(e) => {
            eprintln!("{e}");
            exit(28);
        }
        Ok(parsed) => {
            print_parsed(&srca, parsed.as_ref());
        }
    }
}

const COLOR_COMMENT: Color = Color::BrightBlack;
const COLOR_VARIABLE: Color = Color::Green;
const COLOR_VARIABLE_REF: Color = Color::Green;
const COLOR_IF: Color = Color::Red;
const COLOR_IF_WITH_ELSE: Color = Color::Red;
const COLOR_LOOP: Color = Color::Red;
const COLOR_TUPLE: Color = Color::Blue;
const COLOR_OBJECT: Color = Color::Blue;
const COLOR_VALUE: Color = Color::Cyan;
const COLOR_AS_TYPE: Color = Color::BrightBlack;
const COLOR_CUSTOM_TYPE: Color = Color::BrightBlack;
const COLOR_UNKNOWN: Color = Color::White;

fn print_parsed(srca: &Arc<Source>, parsed: &dyn mers_lib::program::parsed::MersStatement) {
    let mut sections = vec![(COLOR_UNKNOWN, srca.src_og().len())];
    build_print(&mut sections, srca, parsed);
    for (start, comment) in srca.comments() {
        let end = start + comment.len();
        build_print_insert_color(&mut sections, COLOR_COMMENT, *start, end);
    }
    let src = srca.src_og();
    let mut i = 0;
    for (clr, end) in sections {
        print!("{}", src[i..end].color(clr));
        i = end;
    }
    println!();
}
fn build_print(
    sections: &mut Vec<(Color, usize)>,
    srca: &Arc<Source>,
    parsed: &dyn mers_lib::program::parsed::MersStatement,
) {
    let any = parsed.as_any();
    let clr = if let Some(v) = any.downcast_ref::<mers_lib::program::parsed::variable::Variable>() {
        if v.is_ref {
            COLOR_VARIABLE_REF
        } else {
            COLOR_VARIABLE
        }
    } else if let Some(v) = any.downcast_ref::<mers_lib::program::parsed::r#if::If>() {
        if v.on_false.is_some() {
            COLOR_IF_WITH_ELSE
        } else {
            COLOR_IF
        }
    } else if let Some(_) = any.downcast_ref::<mers_lib::program::parsed::r#loop::Loop>() {
        COLOR_LOOP
    } else if let Some(_) = any.downcast_ref::<mers_lib::program::parsed::tuple::Tuple>() {
        COLOR_TUPLE
    } else if let Some(_) = any.downcast_ref::<mers_lib::program::parsed::object::Object>() {
        COLOR_OBJECT
    } else if let Some(_) = any.downcast_ref::<mers_lib::program::parsed::value::Value>() {
        COLOR_VALUE
    } else if let Some(_) = any.downcast_ref::<mers_lib::program::parsed::as_type::AsType>() {
        COLOR_AS_TYPE
    } else if let Some(_) = any.downcast_ref::<mers_lib::program::parsed::custom_type::CustomType>()
    {
        COLOR_CUSTOM_TYPE
    } else {
        COLOR_UNKNOWN
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
    sections: &mut Vec<(Color, usize)>,
    clr: Color,
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
