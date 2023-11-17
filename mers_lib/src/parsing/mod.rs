use std::{fmt::Debug, path::PathBuf, sync::Arc};

use crate::{
    errors::{CheckError, SourcePos},
    program::{self, parsed::block::Block},
};

pub mod statements;
pub mod types;

pub fn parse(src: &mut Source) -> Result<Box<dyn program::parsed::MersStatement>, CheckError> {
    let pos_in_src = src.get_pos();
    let statements = statements::parse_multiple(src, "")?;
    let block = Block {
        pos_in_src: (pos_in_src, src.get_pos()).into(),
        statements,
    };
    Ok(Box::new(block))
}

pub struct Source {
    src_from: SourceFrom,
    src_raw_len: usize,
    src_og: String,
    src: String,
    /// (start, content) of each comment, including start/end (//, /* and */), but NOT newline after //
    comments: Vec<(usize, String)>,
    i: usize,
    sections: Vec<SectionMarker>,
}
impl Clone for Source {
    fn clone(&self) -> Self {
        Self {
            src_from: self.src_from.clone(),
            src_raw_len: self.src_raw_len,
            src_og: self.src_og.clone(),
            src: self.src.clone(),
            comments: self.comments.clone(),
            i: self.i,
            sections: vec![],
        }
    }
}
impl Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Src: {:?}", self.src_from)
    }
}
#[derive(Clone, Debug)]
pub enum SourceFrom {
    File(PathBuf),
    Unspecified,
}
impl Source {
    pub fn new_from_file(path: PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        Ok(Self::new(SourceFrom::File(path), content))
    }
    pub fn new_from_string(source: String) -> Self {
        Self::new(SourceFrom::Unspecified, source)
    }
    pub fn new(src_from: SourceFrom, source: String) -> Self {
        let mut src = String::with_capacity(source.len());
        let mut comment = (0, String::new());
        let mut comments = Vec::new();
        let mut chars = source.char_indices().peekable();
        let mut in_comment = None;
        loop {
            if let Some((i, ch)) = chars.next() {
                match in_comment {
                    Some(false) => {
                        if ch == '\n' {
                            src.push('\n');
                            in_comment = None;
                            comments.push((
                                comment.0,
                                std::mem::replace(&mut comment.1, String::new()),
                            ));
                        } else {
                            comment.1.push(ch);
                        }
                    }
                    Some(true) => {
                        comment.1.push(ch);
                        if ch == '*' && matches!(chars.peek(), Some((_, '/'))) {
                            chars.next();
                            comment.1.push('/');
                            in_comment = None;
                            comments.push((
                                comment.0,
                                std::mem::replace(&mut comment.1, String::new()),
                            ));
                        }
                    }
                    None => match ch {
                        '\\' if matches!(chars.peek(), Some((_, '/'))) => {
                            chars.next();
                            src.push('/');
                        }
                        '/' if matches!(chars.peek(), Some((_, '/'))) => {
                            chars.next();
                            in_comment = Some(false);
                            comment.0 = i;
                            comment.1.push('/');
                            comment.1.push('/');
                        }
                        '/' if matches!(chars.peek(), Some((_, '*'))) => {
                            chars.next();
                            in_comment = Some(true);
                            comment.0 = i;
                            comment.1.push('/');
                            comment.1.push('*');
                        }
                        _ => src.push(ch),
                    },
                }
            } else {
                break;
            }
        }
        Self {
            src_from,
            src_raw_len: source.len(),
            src_og: source,
            src,
            comments,
            i: 0,
            sections: vec![],
        }
    }

    pub fn src(&self) -> &String {
        &self.src
    }
    pub fn comments(&self) -> &Vec<(usize, String)> {
        &self.comments
    }

    pub fn skip_whitespace(&mut self) {
        if let Some(i) = self.src[self.i..].char_indices().find_map(|(i, ch)| {
            if !ch.is_whitespace() {
                Some(i)
            } else {
                None
            }
        }) {
            self.i += i;
        } else {
            self.i = self.src.len();
        }
    }

    fn end_sections(&mut self) {
        let pos = self.get_pos();
        for section in self.sections.iter_mut() {
            section.checked_end(pos);
        }
    }

    pub fn peek_char(&self) -> Option<char> {
        self.src[self.i..].chars().next()
    }
    pub fn next_char(&mut self) -> Option<char> {
        self.end_sections();
        let ch = self.src[self.i..].chars().next()?;
        self.i += ch.len_utf8();
        Some(ch)
    }
    fn word_splitter(ch: char) -> bool {
        ch.is_whitespace() || ".,;[](){}".contains(ch)
    }
    pub fn peek_word(&self) -> &str {
        self.src[self.i..]
            .split(Self::word_splitter)
            .next()
            .unwrap_or("")
    }
    pub fn next_word(&mut self) -> &str {
        self.end_sections();
        let word = self.src[self.i..]
            .split(Self::word_splitter)
            .next()
            .unwrap_or("");
        self.i += word.len();
        word
    }
    pub fn peek_line(&self) -> &str {
        self.src[self.i..].lines().next().unwrap_or("")
    }
    pub fn next_line(&mut self) -> &str {
        self.end_sections();
        let line = self.src[self.i..].lines().next().unwrap_or("");
        self.i += line.len();
        line
    }

    pub fn get_pos(&self) -> SourcePos {
        SourcePos(self.i)
    }
    pub fn set_pos(&mut self, new: SourcePos) {
        self.i = new.0
    }

    /// Returns a SectionMarker which, when dropped, indicates the end of this section.
    /// Useful for debugging the parser.
    pub fn section_begin(&mut self, section: String) -> Arc<String> {
        #[cfg(debug_assertions)]
        println!("[mers:parse] Section begin: {}", &section);
        let arc = Arc::new(section);
        self.sections.push(SectionMarker {
            section: Arc::clone(&arc),
            start: self.get_pos(),
            end: None,
        });
        arc
    }
    pub fn sections(&self) -> &Vec<SectionMarker> {
        &self.sections
    }

    pub fn get_line_start(&self, pos: usize) -> usize {
        self.src[0..pos].rfind("\n").map(|i| i + 1).unwrap_or(0)
    }
    pub fn get_line_end(&self, pos: usize) -> usize {
        // TODO: If the newline is preceded by `\r`s, remove those too since they are part of the newline `\r\n` sequence
        self.src[pos..]
            .find("\n")
            .map(|i| i + pos)
            .unwrap_or(self.src.len())
    }

    pub fn format(&self, insertions: &Vec<(usize, bool, String)>) -> String {
        let mut o = String::with_capacity(self.src_raw_len);
        let mut insertions = insertions.iter().peekable();
        let mut comments = self.comments.iter().peekable();
        for (i, ch) in self.src.char_indices() {
            let insert = if let Some((index, pre, _)) = insertions.peek() {
                if *index <= i {
                    Some(*pre)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some((index, comment)) = comments.peek() {
                if *index == i {
                    comments.next();
                    o.push_str(comment);
                }
            }
            if let Some(true) = insert {
                o.push_str(&insertions.next().unwrap().2);
            }
            o.push(ch);
            if let Some(false) = insert {
                o.push_str(&insertions.next().unwrap().2);
            }
        }
        o
    }

    pub fn src_from(&self) -> &SourceFrom {
        &self.src_from
    }

    pub fn pos_in_og(&self, mut pos: usize, inclusive: bool) -> usize {
        for (start, comment) in &self.comments {
            if *start < pos || (inclusive && *start == pos) {
                pos += comment.len();
            } else {
                break;
            }
        }
        pos
    }
    pub fn src_og(&self) -> &String {
        &self.src_og
    }
}

impl Drop for Source {
    fn drop(&mut self) {
        self.end_sections()
    }
}

/// Returned from Source::begin(), this indicates that parsing of
/// a certain section has not yet finished.
/// Once this is dropped, a section is considered done.
pub struct SectionMarker {
    section: Arc<String>,
    start: SourcePos,
    end: Option<SourcePos>,
}
impl SectionMarker {
    pub fn section(&self) -> &str {
        &self.section
    }
    pub fn start(&self) -> &SourcePos {
        &self.start
    }
    pub fn end(&self) -> &Option<SourcePos> {
        &self.end
    }
    /// If this is the only remaining SectionMarker for this section,
    /// this method sets its `end` property.
    fn checked_end(&mut self, end: SourcePos) {
        if self.end.is_none() {
            if Arc::strong_count(&self.section) == 1 {
                self.end = Some(end);
                #[cfg(debug_assertions)]
                println!("[mers:parse] Section end  : {}", &self.section);
            }
        }
    }
}
