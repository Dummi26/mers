use std::{
    fmt::Display,
    ops::{Index, Range, RangeFrom, RangeTo},
    path::PathBuf,
};

pub struct File {
    path: PathBuf,
    data: String,
    chars: Vec<(usize, char)>,
    pos: FilePosition,
}
#[derive(Clone, Copy)]
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
    pub fn new(data: String, path: PathBuf) -> Self {
        let mut chs = data.chars();
        let mut data = String::with_capacity(data.len());
        loop {
            match chs.next() {
                Some('\\') => match chs.next() {
                    // backslash can escape these characters:
                    Some('\n') => data.push('\\'),
                    // backshash invalidates comments, so \// will just be //.
                    Some('/') => data.push('/'),
                    // backslash does nothing otherwise.
                    Some(ch) => {
                        data.push('\\');
                        data.push(ch);
                    }
                    None => data.push('\\'),
                },
                Some('/') => match chs.next() {
                    Some('/') => loop {
                        match chs.next() {
                            Some('\n') | None => break,
                            _ => (),
                        }
                    },
                    Some('*') => loop {
                        match chs.next() {
                            Some('*') => {
                                if let Some('/') = chs.next() {
                                    break;
                                }
                            }
                            None => break,
                            _ => (),
                        }
                    },
                    Some(ch) => {
                        data.push('/');
                        data.push(ch);
                    }
                    None => {
                        data.push('/');
                        break;
                    }
                },
                Some(ch) => data.push(ch),
                None => break,
            }
        }
        let chars = data.char_indices().collect();
        Self {
            path,
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
    pub fn path(&self) -> &PathBuf {
        &self.path
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
    pub fn next_line(&mut self) -> String {
        let mut o = String::new();
        for ch in self {
            if ch == '\n' {
                break;
            } else {
                o.push(ch);
            }
        }
        o
    }
    pub fn peek(&self) -> Option<char> {
        match self.chars.get(self.pos.current_char_index) {
            Some((_, c)) => Some(*c),
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
