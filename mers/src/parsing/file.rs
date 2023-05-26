use std::{
    fmt::Display,
    ops::{Index, Range, RangeFrom, RangeTo},
    path::PathBuf,
};

pub struct File {
    path: PathBuf,
    data: String,
    chars: Vec<(usize, char)>,
    // contains the byte indices of all newline characters
    newlines: Vec<usize>,
    pos: FilePosition,
    ppos: FilePosition,
}
#[derive(Clone, Copy, Debug)]
pub struct FilePosition {
    pub current_char_index: usize,
    pub current_line: usize,
    pub current_column: usize,
}
impl Display for FilePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line {}, col. {}",
            self.current_line + 1,
            self.current_column + 1
        )
    }
}

impl File {
    pub fn new(data: String, path: PathBuf) -> Self {
        let data = if data.starts_with("#!") {
            &data[data.lines().next().unwrap().len()..].trim_start()
        } else {
            data.trim_start()
        };
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
                            Some('\n') => {
                                data.push('\n');
                                break;
                            }
                            None => break,
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
        if !data.ends_with('\n') {
            data.push('\n');
        }
        let chars: Vec<_> = data.char_indices().collect();
        let newlines: Vec<_> = chars
            .iter()
            .filter_map(|v| if v.1 == '\n' { Some(v.0) } else { None })
            .collect();
        let pos = FilePosition {
            current_char_index: 0,
            current_line: 0,
            current_column: 0,
        };
        Self {
            path,
            data,
            chars,
            newlines,
            pos,
            ppos: pos,
        }
    }
    pub fn skip_whitespaces(&mut self) {
        loop {
            match self.peek() {
                Some(ch) if ch.is_whitespace() => _ = self.next(),
                _ => break,
            }
        }
    }
    pub fn collect_to_whitespace(&mut self) -> String {
        let mut o = String::new();
        loop {
            if let Some(ch) = self.next() {
                if ch.is_whitespace() {
                    self.set_pos(*self.get_ppos());
                    break;
                }
                o.push(ch);
            } else {
                break;
            }
        }
        o
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_pos(&self) -> &FilePosition {
        &self.pos
    }
    pub fn get_ppos(&self) -> &FilePosition {
        &self.ppos
    }
    pub fn set_pos(&mut self, pos: FilePosition) {
        self.pos = pos;
    }
    pub fn get_char(&self, index: usize) -> Option<char> {
        match self.chars.get(index) {
            Some(v) => Some(v.1),
            None => None,
        }
    }
    pub fn get_line(&self, line_nr: usize) -> Option<&str> {
        if self.newlines.len() > line_nr {
            Some(if line_nr == 0 {
                &self.data[0..self.newlines[0]]
            } else {
                &self.data[self.newlines[line_nr - 1] + 1..self.newlines[line_nr]]
            })
        } else if self.newlines.len() == line_nr {
            Some(if line_nr == 0 {
                self.data.as_str()
            } else {
                &self.data[self.newlines[line_nr - 1] + 1..]
            })
        } else {
            None
        }
    }
    // returns the lines. both from and to are inclusive.
    pub fn get_lines(&self, from: usize, to: usize) -> Option<&str> {
        let start_index = if from == 0 {
            0
        } else if from <= self.newlines.len() {
            self.newlines[from - 1] + 1
        } else {
            return None;
        };
        let end_index = if to == self.newlines.len() {
            self.data.len()
        } else if to < self.newlines.len() {
            self.newlines[to]
        } else {
            return None;
        };
        Some(&self.data[start_index..end_index])
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
        self.ppos = self.pos;
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
                // #[cfg(debug_assertions)]
                // eprint!("{ch}");
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
