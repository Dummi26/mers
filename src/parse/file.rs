use std::{
    fmt::Display,
    ops::{Index, Range, RangeFrom, RangeTo},
};

pub struct File {
    data: String,
    chars: Vec<(usize, char)>,
    pos: FilePosition,
}
pub struct FilePosition {
    current_char_index: usize,
    current_line: usize,
    current_column: usize,
}
impl Display for FilePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line {}, col. {}",
            self.current_line, self.current_column
        )
    }
}

impl File {
    pub fn new(data: String) -> Self {
        let chars = data.char_indices().collect();
        Self {
            data,
            chars,
            pos: FilePosition {
                current_char_index: 0,
                current_line: 0,
                current_column: 0,
            },
        }
    }
    pub fn skip_whitespaces(&mut self) {
        loop {
            match self.chars.get(self.pos.current_char_index) {
                Some(ch) if ch.1.is_whitespace() => _ = self.next(),
                _ => break,
            }
        }
    }
    pub fn get_pos(&self) -> &FilePosition {
        &self.pos
    }
    pub fn set_pos(&mut self, pos: FilePosition) {
        self.pos = pos;
    }
    pub fn get_line(&self) -> usize {
        self.pos.current_line
    }
    pub fn get_column(&self) -> usize {
        self.pos.current_column
    }
    pub fn get_char_index(&self) -> usize {
        self.pos.current_char_index
    }
    pub fn get_char(&self, index: usize) -> Option<char> {
        match self.chars.get(index) {
            Some(v) => Some(v.1),
            None => None,
        }
    }
}

impl Iterator for File {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        let o = self.chars.get(self.pos.current_char_index);
        self.pos.current_char_index += 1;
        match o {
            Some((_, ch)) => {
                match *ch {
                    '\n' => {
                        self.pos.current_line += 1;
                        self.pos.current_column = 0;
                    }
                    _ => self.pos.current_column += 1,
                }
                #[cfg(debug_assertions)]
                eprint!("{ch}");
                Some(*ch)
            }
            None => None,
        }
    }
}

impl Index<Range<usize>> for File {
    type Output = str;
    fn index(&self, index: Range<usize>) -> &Self::Output {
        if let Some((start, _)) = self.chars.get(index.start) {
            if let Some((end, _)) = self.chars.get(index.end) {
                &self.data[*start..*end]
            } else {
                &self.data[*start..]
            }
        } else {
            ""
        }
    }
}
impl Index<RangeFrom<usize>> for File {
    type Output = str;
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        if let Some((start, _)) = self.chars.get(index.start) {
            &self.data[*start..]
        } else {
            ""
        }
    }
}
impl Index<RangeTo<usize>> for File {
    type Output = str;
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        if let Some((end, _)) = self.chars.get(index.end) {
            &self.data[..*end]
        } else {
            ""
        }
    }
}
