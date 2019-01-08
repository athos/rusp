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

    fn skip_whitespaces(&mut self) -> Option<char> {
        loop {
            if let x@Some(_) = self.peek_char() {
                if !x.unwrap().is_whitespace() {
                    return x
                }
                self.clear();
            } else {
                return None
            }
        }
    }

    fn read_while(&mut self, f: impl FnMut(&char) -> bool) -> String {
        let c = self.peek_char().unwrap();
        self.clear();
        iter::once(c).chain((&mut self.iter).take_while(f)).collect::<String>()
    }

    fn read_number(&mut self) -> Object {
        let ds = self.read_while(|c| c.is_digit(10));
        Object::Number(ds.parse::<i32>().unwrap())
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
fn reader_test() {
    let mut reader = Reader::new("-123".chars());
    let v = reader.next().unwrap();
    assert_eq!(Object::Number(-123), v);

    let mut reader = Reader::new("set!".chars());
    let v = reader.next().unwrap();
    assert_eq!(Object::Symbol("set!".to_string()), v);

    let mut reader = Reader::new("(1 2 3 )".chars());
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
