use std::char;
use std::rc::Rc;
use crate::error::{Error, error};
use crate::object::Object;

struct ReaderIterator<I: Iterator<Item = char>> {
    iter: I,
    peek: Option<char>
}

impl <I: Iterator<Item = char>> ReaderIterator<I> {
    fn new(iter: I) -> Self {
        ReaderIterator { iter, peek: None }
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
                    Object::Cons(Rc::new(e), Rc::new(acc))
                })
            }
            elems.push(self.next().unwrap());
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for ReaderIterator<I> {
    type Item = Object;

    fn next(&mut self) -> Option<Object> {
        match self.skip_whitespaces()? {
            '-' | '0' ... '9' => Some(self.read_number()),
            '(' => Some(self.read_list()),
            _ => Some(self.read_symbol())
        }
    }
}

pub fn read<'a>(iter: impl 'a + Iterator<Item = char>) -> impl 'a + Iterator<Item = Object> {
    ReaderIterator::new(iter)
}

pub fn read_string(str: &str) -> Option<Object> {
    read(str.chars()).next()
}

#[test]
fn reader_internal_test() {
    let mut r = ReaderIterator::new("123abc".chars());
    let s = r.read_while(|c| c.is_digit(10));
    assert_eq!(s, "123".to_string());
    assert_eq!(r.peek, Some('a'));

    let mut r = ReaderIterator::new("123abc".chars());
    r.drop_while(|c| c.is_digit(10));
    assert_eq!(r.iter.collect::<String>(), "bc".to_string());
    assert_eq!(r.peek, Some('a'));
}

#[test]
fn reader_test() {
    let read = |s| read_string(s).unwrap();

    assert_eq!(read("t"), Object::T);
    assert_eq!(read("nil"), Object::Nil);
    assert_eq!(read("-123"), Object::Number(-123));
    assert_eq!(read("hello-world!"), Object::Symbol("hello-world!".to_string()));
    assert_eq!(read("(1 2 3)"), Object::Cons(
        Rc::new(Object::Number(1)),
        Rc::new(Object::Cons(
            Rc::new(Object::Number(2)),
            Rc::new(Object::Cons(
                Rc::new(Object::Number(3)),
                Rc::new(Object::Nil)
            ))
        ))
    ));
}
