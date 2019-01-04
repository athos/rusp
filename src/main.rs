mod rusp;

use rusp::object;

fn main() {
    let obj = object::cons(
        object::symbol("+"),
        object::cons(
            object::number(1),
            object::cons(object::number(2), object::nil)
        )
    );
    println!("{}", obj);
}
