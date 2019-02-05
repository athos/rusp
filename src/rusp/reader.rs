use std::char;
use std::error;
use std::fmt;
use std::iter;
use rusp::object::Object;

pub struct Reader<I: Iterator<Item = char>> {
    iter: I,
    peek: Option<char>
}

#[derive(Debug, Clone)]
pub struct ReaderError {
    message: String
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl error::Error for ReaderError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl <I: Iterator<Item = char>> Reader<I> {
    pub fn new(iter: I) -> Self {
        Reader { iter, peek: None }
    }

    fn next_char(&mut self) -> Option<char> {
        self.iter.next()
    }

    fn peek_char(&mut self) -> Option<char> {
        if self.peek == None {
            self.peek = self.next_char()
        }
        self.peek
    }

    fn clear(&mut self) {
        self.peek = None;
    }

    fn read_while(&mut self, mut f: impl FnMut(&char) -> bool) -> String {
        let mut ret = String::new();
        if let Some(c) = self.peek {
            if !f(&c) {
                self.peek = Some(c);
                return ret;
            }
            ret.push(c);
            self.clear();
        }
        for c in self.iter.by_ref() {
            if f(&c) {
                ret.push(c);
            } else {
                self.peek = Some(c);
                break;
            }
        }
        ret
    }

    fn drop_while(&mut self, mut f: impl FnMut(&char) -> bool) {
        if let Some(c) = self.peek {
            if !f(&c) {
                return;
            }
        }
        for c in self.iter.by_ref() {
            if !f(&c) {
                self.peek = Some(c);
                break;
            }
        }
    }

    fn skip_whitespaces(&mut self) -> Option<char> {
        self.drop_while(|c| c.is_whitespace());
        self.peek_char()
    }

    fn read_number(&mut self) -> Object {
        let negative = self.peek.unwrap_or('0') == '-';
        if negative {
            self.clear();
        }
        let ds = self.read_while(|c| c.is_digit(10));
        let num = ds.parse::<i32>().unwrap();
        Object::Number(if negative { -num } else { num })
    }

    fn read_symbol(&mut self) -> Object {
        let name = self.read_while(|c| {
            match c {
                '(' | ')' | '\'' | '"' | ',' => false,
                _ => !c.is_whitespace()
            }
        });
        match name.as_str() {
            "t" => Object::T,
            "nil" => Object::Nil,
            _ => Object::Symbol(name)
        }
    }

    fn read_list(&mut self) -> Object {
        let mut elems: Vec<Object> = vec![];
        self.clear();
        loop {
            let c = self.peek_char().unwrap();
            if c == ')' {
                self.clear();
                return elems.into_iter().rev().fold(Object::Nil, |acc, e| {
                    Object::Cons {
                        car: Box::new(e),
                        cdr: Box::new(acc)
                    }
                })
            }
            elems.push(self.next().unwrap());
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for Reader<I> {
    type Item = Object;

    fn next(&mut self) -> Option<Object> {
        match self.skip_whitespaces()? {
            '-' | '0' ... '9' => Some(self.read_number()),
            '(' => Some(self.read_list()),
            _ => Some(self.read_symbol())
        }
    }
}

#[test]
fn reader_internal_test() {
    let mut reader = Reader::new("123abc".chars());
    let s = reader.read_while(|c| c.is_digit(10));
    assert_eq!(s, "123".to_string());
    assert_eq!(reader.peek, Some('a'));

    let mut reader = Reader::new("123abc".chars());
    reader.drop_while(|c| c.is_digit(10));
    assert_eq!(reader.iter.collect::<String>(), "bc".to_string());
    assert_eq!(reader.peek, Some('a'));
}

#[test]
fn reader_test() {
    let mut reader = Reader::new("t".chars());
    let v = reader.next().unwrap();
    assert_eq!(Object::T, v);

    let mut reader = Reader::new("nil".chars());
    let v = reader.next().unwrap();
    assert_eq!(Object::Nil, v);

    let mut reader = Reader::new("-123".chars());
    let v = reader.next().unwrap();
    assert_eq!(Object::Number(-123), v);

    let mut reader = Reader::new("hello-world!".chars());
    let v = reader.next().unwrap();
    assert_eq!(Object::Symbol("hello-world!".to_string()), v);

    let mut reader = Reader::new("(1 2 3)".chars());
    let v = reader.next().unwrap();
    assert_eq!(v, Object::Cons {
        car: Box::new(Object::Number(1)),
        cdr: Box::new(Object::Cons {
            car: Box::new(Object::Number(2)),
            cdr: Box::new(Object::Cons {
                car: Box::new(Object::Number(3)),
                cdr: Box::new(Object::Nil)
            })
        })
    });
}
