use super::*;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_till, take_until, take_while1},
    character::complete::{
        alphanumeric1, char, digit1, multispace0, multispace1, none_of, one_of,
    },
    combinator::{eof, map, not, opt, peek, recognize, rest},
    multi::{fold_many0, many0, many1, many_till, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    FindSubstring, IResult, InputTake,
};

///parse script
pub fn parse(input: &str) -> Result<Vec<Cell>, String> {
    match parse_cells(input) {
        Ok((_, cells)) => Ok(cells),
        Err(e) => Err(format!("{:?}", e)),
    }
}

#[test]
fn test_parse_cells() {
    let input = r#"
        let x = 5;
        let y = "hello";

        task greet {
            echo "hello world"
        }:shell

        runtime dart {
            let version = "3.7.0"
            let path = "path/to/dart.exe"

            task build {
                echo "Building with dart"
                [:path] --version
                [:path] run [:file]
            }:shell
        }:moto

        block developerCredits {
            developed by incredimo for xo.rs
        }:text

        import "math.moto" as math
        "#;

    let result = parse_cells(input).unwrap();
    println!("{}, {:?}", input, result);
}

///parse cells
/// a cell can be a task, a runtime, a block, a variable etc
/// cells can be nested and can contain other cells
pub fn parse_cell(input: &str) -> IResult<&str, Cell> {

    let (input, cell) = delimited(
        ignore_comments_and_spaces,
        alt((
            map(parse_assignment, Cell::Assignment),
            map(parse_task, Cell::Task),
            map(parse_runtime, Cell::Runtime),
            map(parse_block, Cell::Block),
            map(parse_import, Cell::Import),
        )),
        ignore_comments_and_spaces,
    )(input)?;

    Ok((input,cell))


}



pub fn parse_cells(input: &str) -> IResult<&str, Vec<Cell>> {
    many0(parse_cell)(input)
}

#[test]
fn test_parse_package() {
    let input = r#"
        package rust { // this is a comment
            let version = "1.0.0";

            runtime rust {
                let version = "1.0.0";
                let path = "path/to/rust.exe";

                task build {
                    echo "Building with rust"
                    [:path] --version
                    [:path] run [:file]
                }:shell
            }:moto
        }:moto
        "#;

    let (input, result) = parse_package(input).unwrap();
    assert_eq!(
        result,
        Package {
            identifer: Identifier::new("rust"),
            children: vec![
                Cell::Assignment(Assignment::new("version", "1.0.0")),
                Cell::Runtime(Runtime {
                    identifer: Identifier::new("rust"),
                    children: vec![
                        Cell::Assignment(Assignment::new("version", "1.0.0")),
                        Cell::Assignment(Assignment::new("path", "path/to/rust.exe")),
                        Cell::Task(Task {
                            identifer: Identifier::new("build"),
                            body: String::from("echo \"Building with rust\" [:path] --version [:path] run [:file]"),
                            runtime: Identifier::new("shell")
                        })
                    ],
                    runtime: "moto".into()
                })
            ],
            runtime: "moto".into()
        }
    );
}


#[test]
fn test_comments_and_spaces() {
    let input = r#"
        // this is a comment
        /* this is a block comment */
        "#;

    let result = ignore_comments_and_spaces(input).unwrap();
    assert_eq!(result, ("", ""));
}

#[test]
fn test_ignore_comments_everywhere() {
    let input = r#"
    // this is a comment
        package rust { // this is a comment
            let version = "1.0.0";  // this is a comment
            runtime rust { // this is a comment
                let version = "1.0.0"; // this is a comment
                let path = "path/to/rust.exe"; // this is a comment
                task build {
                    echo "Building with rust"
                    [:path] --version 
                    [:path] run [:file] 
                }:shell // this is a comment
            }:moto // this is a comment
        }:moto
        "#;

    let result = parse_package(input).unwrap();
    result.0.println(in_cyan);
    result.1.println(in_yellow);


    let input = r#"
    
// moto script v2.0
// moto scripts are written in a simple language that is easy to understand and write
// a moto script is broken down into a collection of cells. cells are the basic building blocks of a moto script
// a cell can be a package,runtime or a task
// a task is a sequence of commands that are executed in order one line at a time using the runtime specified at its tail


task greet {
    echo "hello world"
}:ps

// tasks can be defined in any runtime , provided the runtime's definition is available to the moto runtime
// let's define a task in the dart runtime
task greet_from_dart {
    print("hello world");
}:dart

// a runtime is a collection of tasks and variables that are used to execute a task
"#;
    let result = parse_cells(input).unwrap();
    result.0.println(in_cyan);
    for cell in result.1 {
        cell.println(in_yellow);
    }



}



// comments can be single line or multi line
// single line comments start with // and end with a newline
// multi line comments start with /* and end with */ and can span multiple lines
// comments can appear anywhere in the script and are ignored by the parser
// comments can also appear at the end of a line and are ignored by the parser
// linebreaks after a comment are also ignored by the parser until a non comment character is encountered
pub fn comment(input: &str) -> IResult<&str, &str> {
    alt((single_line_comment, multi_line_comment))(input)
}

pub fn single_line_comment(input: &str) -> IResult<&str, &str> {

    let (input, _) = tag("//")(input)?;
    let (input, _) = take_till(|c| c == '\n')(input)?;
    let (input, _) = char('\n')(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, ""))
}

pub fn multi_line_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("/*")(input)?;
    let (input, _) = take_until("*/")(input)?;
    let (input, _) = tag("*/")(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, ""))
}

pub fn comments(input: &str) -> IResult<&str, &str> {
    let (input, _) = multispace0(input)?;
    let (input, _) = many0(comment)(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, ""))
}

// captures and ignores optional comments and spaces until end of comment or space
pub fn ignore_comments_and_spaces(input: &str) -> IResult<&str, &str> {
    let (input, _) = multispace0(input)?;
    let (input, _) = opt(comments)(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, ""))
}

pub fn parse_package(input: &str) -> IResult<&str, Package> {

        let (input, _) = ignore_comments_and_spaces(input)?;
        let (input, _) = tag("package")(input)?;
        let (input, _) = multispace1(input)?;
        let (input, identifier) = parse_identifier(input)?;
        let (input, _) = ignore_comments_and_spaces(input)?;
        let (input, _) = char('{')(input)?;
        let (input, _) = ignore_comments_and_spaces(input)?;
    
        // Parse children (assignments or tasks) until the closing tag of the runtime block
        let (input, children) = many0(alt((
            map(parse_assignment, Cell::Assignment),
            map(parse_task, Cell::Task),
            map(parse_runtime, Cell::Runtime),
            map(parse_block, Cell::Block),
            map(parse_import, Cell::Import),

        )))(input)?;
    
        let (input, _) = ignore_comments_and_spaces(input)?;
        let (input, _) = tag("}:")(input)?;
        let (input, runtime) = parse_identifier(input)?;
        let (input, _) = ignore_comments_and_spaces(input)?;
        let (input, _) = opt(eof)(input)?; // Optional EOF to ensure parsing until the end of input
    
        Ok((
            input,
            Package {
                identifer: identifier,
                children,
                runtime,
            },
        ))
    
}


#[test]
fn test_parse_assignment() {
    let input = r#"
        let x = 5; // this is a comment
        let y = "hello";
        let z = true;
        "#;

    let (input, result) = parse_assignment(input).unwrap();
    assert_eq!(
        result,
        Assignment {
            identifier: Identifier ("x".to_string()),
            value: Atom::Number(5.0)
        }
    );

    let (input, result) = parse_assignment(input).unwrap();
    assert_eq!(
        result,
        Assignment {
            identifier: Identifier ("y".to_string()),
            value: Atom::String("hello".to_string())
        }
    );

    let (input, result) = parse_assignment(input).unwrap();
    assert_eq!(
        result,
        Assignment {
            identifier: Identifier::new("z"),
            value: Atom::Boolean(true)
        }
    );
}

pub fn parse_assignment(input: &str) -> IResult<&str, Assignment> {
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = tag("let")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, identifer) = parse_identifier(input)?;
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, value) = parse_atom(input)?;
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = char(';')(input)?;
    Ok((
        input,
        Assignment {
            identifier: identifer,
            value,
        },
    ))
}

#[test]
fn test_parse_task() {
    let input = r#"
        task greet {echo "hello world"}:shell
        "#;

    let (input, result) = parse_task(input).unwrap();
    assert_eq!(
        result,
        Task {
            identifer: Identifier::new("greet"),
            body: String::from("echo \"hello world\""),
            runtime: Identifier::new("shell")
        }
    );

    let input = r#"task greet {
            print("hello ") [:name]
        }:dart"#;

    let (input, result) = parse_task(input).unwrap();

    assert_eq!(
        result,
        Task {
            identifer: Identifier::new("greet"),
            body:   String::from("print(\"hello \") [:name]"),
            runtime: Identifier::new("dart")
        }
    );
}

pub fn parse_task(input: &str) -> IResult<&str, Task> {
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = tag("task")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, identifier) = parse_identifier(input)?;
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = tag("{")(input)?;
    let (input, body) = take_until("}:")(input)?;
    // Ensure that the closing tag is consumed by the task parser
    let (input, _) = tag("}:")(input)?;
    let (input, runtime) = parse_identifier(input)?;

    Ok((
        input,
        Task {
            identifer: identifier,
            body: String::from(body),
            runtime,
        },
    ))
}

#[test]
fn test_parse_runtime() {
    let input = r#"
        runtime dart { // this is a comment
            let version = "3.7.0";
            let path = "path/to/dart.exe";

            task build {
                echo "Building with dart" 
                [:path] --version [:path] 
                run [:file] }:shell 
            }:moto
        "#;

    let (input, result) = parse_runtime(input).unwrap();
    assert_eq!(
        result,
        Runtime {
            identifer: Identifier::new("dart"),
            children: vec![
                Cell::Assignment(Assignment::new("version", "3.7.0")),
             Cell::Assignment(Assignment{
                    identifier:"path".into(),
                    value: "path/to/dart.exe".into()
                }),
                Cell::Task(Task {
                    identifer: Identifier::new("build"),
                    body: String::from("echo \"Building with dart\" [:path] --version [:path] run [:file]".to_string()),
                    runtime: Identifier::new("shell")
                })
            ],
            runtime: "moto".into()
        }
    );
}

pub fn parse_runtime(input: &str) -> IResult<&str, Runtime> {
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = tag("runtime")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, identifier) = parse_identifier(input)?;
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = char('{')(input)?;
    let (input, _) = ignore_comments_and_spaces(input)?;

    // Parse children (assignments or tasks) until the closing tag of the runtime block
    let (input, children) = many0(alt((
        map(parse_assignment, Cell::Assignment),
        map(parse_task, Cell::Task),
    )))(input)?;

    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = tag("}:")(input)?;
    let (input, runtime) = parse_identifier(input)?;
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = opt(eof)(input)?; // Optional EOF to ensure parsing until the end of input

    Ok((
        input,
        Runtime::new(identifier, runtime, children),
    ))
}

#[test]
fn test_parse_block() {
    let input = r#"
        block developerCredits { developed by incredimo for xo.rs }:text // this is a comment
        "#;

    let (input, result) = parse_block(input).unwrap();
    assert_eq!(
        result,
        Block::new("developerCredits", "developed by incredimo for xo.rs", "text")
    );
}

pub fn parse_block(input: &str) -> IResult<&str, Block> {
    //first we will take the outer frame
    let mut task_parser = tuple((
        ignore_comments_and_spaces,
        tag("block"),
        multispace1,
        parse_identifier,
        ignore_comments_and_spaces,
        tag("{"),
        take_until("}:"),
        tag("}:"),
        parse_identifier,
    ));
    let (input, (_, _, _, identifer, _, _, body, _, runtime)) = task_parser(input)?;

    Ok((
        input,
        Block::new( identifer, body, runtime)
    ))
}

#[test]
fn test_parse_import() {
    let input = r#"
        import "math.moto" as math // this is a comment
        "#;

    let (input, result) = parse_import(input).unwrap();
    assert_eq!(
        result,
        Import::new("math.moto", "math")
    );
}

pub fn parse_import(input: &str) -> IResult<&str, Import> {
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = tag("import")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, path) = parse_string(input)?;
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, _) = tag("as")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, alias) = parse_identifier(input)?;
    Ok((input, Import { path, alias }))
}

#[test]
fn test_parse_identifier() {
    let input = r#"x"#;
    let (input, result) = parse_identifier(input).unwrap();
    assert_eq!(
        result,
        "x".into()
    );

    let input = r#"projectName"#;
    let (input, result) = parse_identifier(input).unwrap();
    assert_eq!(
        result,
        Identifier::new("projectName")
    );

    let input = r#"project_name"#;
    let (input, result) = parse_identifier(input).unwrap();
    assert_eq!(
        result,
       "project_name".into()
    );

    let input = r#"
        project_name
        "#;
    let (input, result) = parse_identifier(input).unwrap();
    assert_eq!(
        result,
        "project_name".into()
    );
}

pub fn parse_identifier(input: &str) -> IResult<&str, Identifier> {
    let (input, _) = multispace0(input)?;
    let (input, name) = recognize(pair(
        one_of("_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        many0(one_of(
            "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
        )),
    ))(input)?;
    let (input, _) = multispace0(input)?;
    Ok((
        input,
         name.into()
    ))
}

#[test]
fn test_parse_atom() {
    let input = r#"5"#;
    let (input, result) = parse_atom(input).unwrap();
    assert_eq!(result, Atom::Number(5.0));

    let input = r#""hello""#;
    let (input, result) = parse_atom(input).unwrap();
    assert_eq!(result, Atom::String("hello".to_string()));

    let input = r#"true"#;
    let (input, result) = parse_atom(input).unwrap();
    assert_eq!(result, Atom::Boolean(true));

    let input = r#"[1,2,3]"#;
    let (input, result) = parse_atom(input).unwrap();
    assert_eq!(
        result,
        Atom::Array(Box::new(Array {
            values: vec![Atom::Number(1.0), Atom::Number(2.0), Atom::Number(3.0)]
        }))
    );

    let input = r#"{a:1, b:2}"#;
    let (input, result) = parse_atom(input).unwrap();
    assert_eq!(
        result,
        Atom::Object(Box::new(Object {
            values: vec![
                ("a".to_string(), Atom::Number(1.0)),
                ("b".to_string(), Atom::Number(2.0))
            ]
        }))
    );

    // let input = r#"5 + 5"#;
    // let (input, result) = parse_atom(input).unwrap();
    // assert_eq!(result, Atom::BinaryOperation(Box::new(BinaryOperation {
    //     left: Atom::Number(5.0),
    //     operator: Operator { value: "+".to_string() },
    //     right: Atom::Number(5.0)
    // })));

    // let input = r#"x"#;
    // let (input, result) = parse_atom(input).unwrap();
    // assert_eq!(result, Atom::Variable(Box::new(Variable {
    //     identifier: Identifier { name: "x".to_string() },
    //     default: None
    // })));

    // let input = r#"print("hello world")"#;
    // let (input, result) = parse_atom(input).unwrap();
    // assert_eq!(result, Atom::Function(Box::new(Function {
    //     identifier: Identifier { name: "print".to_string() },
    //     arguments: vec![Atom::String("hello world".to_string())]
    // })));
}

pub fn parse_atom(input: &str) -> IResult<&str, Atom> {
    alt((
        map(parse_number, Atom::Number),
        map(parse_string, Atom::String),
        map(parse_boolean, Atom::Boolean),
        map(parse_array, |x| Atom::Array(Box::new(x))),
        map(parse_object, |x| Atom::Object(Box::new(x))),
        map(parse_binary_operation, |x| {
            Atom::BinaryOperation(Box::new(x))
        }),
        // map(parse_variable_atom, |x| Atom::Variable(Box::new(x))),
        map(parse_function, |x| Atom::Function(Box::new(x))),
    ))(input)
}

pub fn parse_variable_atom(input: &str) -> IResult<&str, Variable> {
    let (input, _) = char('[')(input)?;
    let (input, identifier) = parse_identifier(input)?;
    let (input, default) = opt(preceded(char('='), parse_atom))(input)?;
    let (input, _) = char(']')(input)?;
    Ok((
        input,
        Variable::new(identifier, default),
    ))
}

pub fn parse_binary_operator(input: &str) -> IResult<&str, Operator> {
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, operator) = alt((
        tag("+"),
        tag("-"),
        tag("*"),
        tag("/"),
        tag("=="),
        tag("!="),
        tag(">"),
        tag("<"),
        tag(">="),
        tag("<="),
        tag("&&"),
        tag("||"),
    ))(input)?;
    let (input, _) = ignore_comments_and_spaces(input)?;
    Ok((
        input,
        Operator {
            value: operator.to_string(),
        },
    ))
}

#[test]
fn test_parse_number() {
    let input = r#"5"#;
    let (input, result) = parse_number(input).unwrap();
    assert_eq!(result, 5.0);
}

pub fn parse_number(input: &str) -> IResult<&str, f64> {
    let (input, number) = digit1(input)?;
    Ok((input, number.parse().unwrap()))
}

#[test]
fn test_parse_string() {
    let input = r#""hello""#;
    let (input, result) = parse_string(input).unwrap();
    assert_eq!(result, "hello".to_string());
}

pub fn parse_string(input: &str) -> IResult<&str, String> {
    let (input, _) = char('"')(input)?;
    let (input, string) = is_not("\"")(input)?;
    let (input, _) = char('"')(input)?;
    Ok((input, string.to_string()))
}

#[test]
fn test_parse_boolean() {
    let input = r#"true"#;
    let (input, result) = parse_boolean(input).unwrap();
    assert_eq!(result, true);

    let input = r#"false"#;
    let (input, result) = parse_boolean(input).unwrap();
    assert_eq!(result, false);
}

pub fn parse_boolean(input: &str) -> IResult<&str, bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(input)
}

#[test]
fn test_parse_array() {
    let input = r#"[1,2,3]"#;
    let (input, result) = parse_array(input).unwrap();
    assert_eq!(
        result,
        Array {
            values: vec![Atom::Number(1.0), Atom::Number(2.0), Atom::Number(3.0)]
        }
    );
}

pub fn parse_array(input: &str) -> IResult<&str, Array> {
    let (input, _) = char('[')(input)?;
    let (input, values) = separated_list0(char(','), parse_atom)(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, Array { values }))
}

#[test]
fn test_parse_object() {
    let input = r#"{a:1, b:2}"#;
    let (input, result) = parse_object(input).unwrap();
    assert_eq!(
        result,
        Object {
            values: vec![
                ("a".to_string(), Atom::Number(1.0)),
                ("b".to_string(), Atom::Number(2.0))
            ]
        }
    );
}

pub fn parse_object(input: &str) -> IResult<&str, Object> {
    let (input, _) = char('{')(input)?;
    let (input, values) = separated_list0(char(','), parse_key_value_pair)(input)?;
    let (input, _) = char('}')(input)?;
    Ok((input, Object { values }))
}

pub fn parse_key_value_pair(input: &str) -> IResult<&str, (String, Atom)> {
    let (input, _) = ignore_comments_and_spaces(input)?;
    let (input, key) = is_not(":")(input)?;
    let (input, _) = char(':')(input)?;
    let (input, value) = parse_atom(input)?;
    Ok((input, (key.to_string(), value)))
}

#[test]
fn test_parse_binary_operation() {
    let input = r#"5 + 5"#;
    let (input, result) = parse_binary_operation(input).unwrap();
    assert_eq!(
        result,
        BinaryOperation {
            left: Atom::Number(5.0),
            operator: Operator {
                value: "+".to_string()
            },
            right: Atom::Number(5.0)
        }
    );
}

pub fn parse_binary_operation(input: &str) -> IResult<&str, BinaryOperation> {
    let (input, left) = parse_atom(input)?;
    let (input, _) = multispace1(input)?;
    let (input, operator) = parse_operator(input)?;
    let (input, _) = multispace1(input)?;
    let (input, right) = parse_atom(input)?;
    Ok((
        input,
        BinaryOperation {
            left,
            operator,
            right,
        },
    ))
}

#[test]
fn test_parse_operator() {
    let input = r#"+"#;
    let (input, result) = parse_operator(input).unwrap();
    assert_eq!(
        result,
        Operator {
            value: "+".to_string()
        }
    );
}

pub fn parse_operator(input: &str) -> IResult<&str, Operator> {
    let (input, operator) = alt((
        tag("+"),
        tag("-"),
        tag("*"),
        tag("/"),
        tag("=="),
        tag("!="),
        tag(">"),
        tag("<"),
        tag(">="),
        tag("<="),
        tag("&&"),
        tag("||"),
    ))(input)?;
    Ok((
        input,
        Operator {
            value: operator.to_string(),
        },
    ))
}

#[test]
fn test_parse_variable() {
    let input = r#"[:name]"#;
    let (input, result) = parse_variable(input).unwrap();
    assert_eq!(
        result,
        Variable::new("name", None)
    );

    let input = r#"[:x=5]"#;
    let (input, result) = parse_variable(input).unwrap();
    assert_eq!(
        result,
        Variable::new("x", Atom::Number(5.0))
    );

    let input = r#"[:x="hello"]"#;
    let (input, result) = parse_variable(input).unwrap();
    assert_eq!(
        result,
        Variable::new("x", "hello")
    );

    let input = r#"[:x=true]"#;
    let (input, result) = parse_variable(input).unwrap();
    assert_eq!(
        result,
        Variable::new("x", true)
    );
}

pub fn parse_variable(input: &str) -> IResult<&str,  Variable> {
    let (input, _) = tag("[:")(input)?;
    let (input, identifier) = parse_identifier(input)?;
    let (input, default) = opt(preceded(char('='), parse_atom))(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((
        input,
         Variable::new(identifier, default)
    ))
}

#[test]
fn test_parse_function() {
    let input = r#"[:print("hello world")]"#;
    let (input, result) = parse_function(input).unwrap();
    assert_eq!(
        result,
        Function::new("print", vec![Atom::String("hello world".to_string())])
    );
}

pub fn parse_function(input: &str) -> IResult<&str, Function> {
    let (input, _) = tag("[:")(input)?;
    let (input, identifier) = parse_identifier(input)?;
    let (input, arguments) =
        delimited(char('('), separated_list0(char(','), parse_atom), char(')'))(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((
        input,
        Function {
            identifier,
            arguments,
        },
    ))
}



pub fn parse_text(input: &str) -> IResult<&str, String> {
    //if the input contains "[:", then we take everything before the "[:"
    let (input, text) = alt((take_until("[:"), rest))(input)?;
    Ok((input, text.to_string()))
}
