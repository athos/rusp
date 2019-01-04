mod rusp;

use rusp::object::{self, Object};

fn main() {
    let obj = object::cons(
        object::symbol("+"),
        object::cons(
            Object::Number(1),
            object::cons(Object::Number(2), Object::Nil)
        )
    );
    println!("{}", obj);
}
