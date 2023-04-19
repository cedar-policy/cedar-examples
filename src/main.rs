fn main() {
    println!("Hello, world!");
}


fn add_numbers(x : i32, y : i32) -> i32 {
    x + y + 2
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn simple() {
        assert_eq!(add_numbers(1, 2), 3);
    }
}
