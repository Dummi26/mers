use std::sync::Arc;

use crate::program;

pub mod errors;
pub mod statements;
pub mod types;

pub fn parse(src: &mut Source) -> Result<Box<dyn program::parsed::MersStatement>, ()> {
    Ok(Box::new(statements::parse_block(src)?))
}

pub struct Source {
    src: String,
    i: usize,
    sections: Vec<SectionMarker>,
}
impl Source {
    pub fn new(src: String) -> Self {
        Self {
            src,
            i: 0,
            sections: vec![],
        }
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
        ch.is_whitespace() || ".,;)}".contains(ch)
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

#[derive(Clone, Copy)]
pub struct SourcePos(usize);
impl SourcePos {
    fn diff(&self, rhs: &Self) -> usize {
        rhs.0 - self.0
    }
}
