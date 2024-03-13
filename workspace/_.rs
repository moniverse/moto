    fn main() {
        // lets try a more complex example
        let x = 5;
        let y = "hello";
        let z = true;
        //print in red
        println!("\x1b[31mthis is being printed from rust using \x1b[32mmoto's \x1b[33msuper smart \x1b[34mcompiler \x1b[0m");
        println!("x: {}", x);
        println!("y: {}", y);
        println!("z: {}", z);
    }
