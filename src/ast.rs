use std::fmt::Display;

pub use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
//cell is a basic unit of a script
//it can be a task, a runtime, a block, a variable etc
//cells can be nested and can contain other cells

pub enum Cell {
    // assignments are used to define variables
    //e.g `let x = 5;`
    Assignment(Assignment),
    //tasks are used to define a sequence of commands
    //e.g `task greet { echo "hello world" }:shell`
    // #[display(fmt = "task [:identifer] {{ [:body] }}:[:runtime]")]
    Task(Task),
    //runtimes are used to define a runtime for a specific language
    //e.g `runtime dart { let version = "3.7.0" }:shell`
    Runtime(Runtime),
    //blocks are used to define reusable blocks of code
    //e.g `block developerCredits { developed by incredimo for xo.rs }:text`
    Block(Block),
    //imports are used to import code from other cells
    //e.g `import "math.moto" as math`
    #[display(fmt = "import [:path] as [:alias]")]
    Import(Import),
    ///package is used to define a package
    /// a package has multiple cells    
    Package(Package),
}

impl Cell {
    pub fn identifier(&self) -> Option<Identifier> {
        match self {
            Cell::Assignment(assignment) => Some(assignment.identifier.clone()),
            Cell::Task(task) => Some(task.identifer.clone()),
            Cell::Runtime(runtime) => Some(runtime.identifer.clone()),
            Cell::Block(block) => Some(block.identifer.clone()),
            Cell::Import(import) => Some(import.alias.clone()),
            Cell::Package(package) => Some(package.identifer.clone()),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Cell::Assignment(assignment) => assignment.identifier.0.clone(),
            Cell::Task(task) => task.identifer.0.clone(),
            Cell::Runtime(runtime) => runtime.identifer.0.clone(),
            Cell::Block(block) => block.identifer.0.clone(),
            Cell::Import(import) => import.alias.0.clone(),
            Cell::Package(package) => package.identifer.0.clone(),
        }
    }

    pub fn get_kind(&self) -> String {
        match self {
            Cell::Assignment(_) => "assignment".to_string(),
            Cell::Task(_) => "task".to_string(),
            Cell::Runtime(_) => "runtime".to_string(),
            Cell::Block(_) => "block".to_string(),
            Cell::Import(_) => "import".to_string(),
            Cell::Package(_) => "package".to_string(),
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            Cell::Assignment(assignment) => format!("let {} = {}", assignment.identifier, assignment.value),
            Cell::Task(task) =>  format!("task {} with runtime {}", task.identifer, task.runtime),
            Cell::Runtime(runtime) =>  format!("runtime {} with runtime {}", runtime.identifer, runtime.runtime),
            Cell::Block(block) =>  format!("block {} with runtime {}", block.identifer, block.runtime),
            Cell::Import(import) =>  format!("import {} as {}", import.path, import.alias),
            Cell::Package(package) =>  format!("package {}", package.identifer),
        }
    }

    pub fn identifier_is(&self, name: impl Into<String>) -> bool {
        match self {
            Cell::Assignment(assignment) => assignment.identifier.matches(name),
            Cell::Task(task) => task.identifier_is(name),
            Cell::Runtime(runtime) => runtime.identifier_is(name),
            Cell::Block(block) => block.identifier_is(name),
            Cell::Import(import) => import.alias.matches(name),
            Cell::Package(package) => package.identifier_is(name),
        }
    }

    pub fn get_runtime(&self) -> Option<Identifier> {
        match self {
            Cell::Task(task) => Some(task.runtime.clone()),
            Cell::Runtime(runtime) => Some(runtime.runtime.clone()),
            Cell::Block(block) => Some(block.runtime.clone()),
            _ => None,
        }
    }

    pub fn get_body(&self) -> Option<InterpolatedString> {
        match self {
            Cell::Task(task) => Some(task.body.clone()),
            Cell::Block(block) => Some(block.body.clone()),
            _ => None,
        }
    }

    pub fn assignment(identifier: impl Into<String>, value: impl Into<Atom>) -> Self {
        Cell::Assignment(Assignment {
            identifier: Identifier(identifier.into()),
            value: value.into(),
        })
    }

    pub fn task(identifer: impl Into<String>, body: impl Into<InterpolatedString>, runtime: impl Into<String>) -> Self {
        Cell::Task(Task {
            identifer: Identifier(identifer.into()),
            body: body.into(),
            runtime: Identifier(runtime.into()),
        })
    }

    pub fn runtime(identifer: impl Into<String>, runtime: impl Into<String>, children: Vec<Cell>) -> Self {
        Cell::Runtime(Runtime {
            identifer: Identifier(identifer.into()),
            children,
            runtime: Identifier(runtime.into()),
        })
    }

    pub fn block(identifer: impl Into<String>, body: impl Into<InterpolatedString>, runtime: impl Into<String>) -> Self {
        Cell::Block(Block {
            identifer: Identifier(identifer.into()),
            body: body.into(),
            runtime: Identifier(runtime.into()),
        })
    }

    pub fn package(identifer: impl Into<String>, children: Vec<Cell>) -> Self {
        Cell::Package(Package {
            identifer: Identifier(identifer.into()),
            children,
            runtime: Identifier("moto".to_string()),
        })
    }



    pub fn import(path: impl Into<String>, alias: impl Into<String>) -> Self {
        Cell::Import(Import {
            path: path.into(),
            alias: Identifier(alias.into()),
        })
    }


}

#[derive(Debug, Clone, PartialEq, Eq)]
///packages are used to define a package
/// a package has multiple cells
/// e.g `package math { let x = 5; let y = 10; }`
/// or `package math { task greet { echo "hello world" }:shell }`
/// or `package math { runtime dart { let version = "3.7.0" }:shell }`
/// or `package math { block developerCredits { developed by incredimo for xo.rs }:text }`
/// or `package math { import "math.moto" as math }`

pub struct Package {
    pub identifer: Identifier,
    pub children: Vec<Cell>,
    pub runtime: Identifier,
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "package {} {{\n", self.identifer.0)?;
        for child in &self.children {
            write!(f, "{}\n", child)?;
        }
        write!(f, "}}")
    }
}

impl Into<Cell> for Package {
    fn into(self) -> Cell {
        Cell::Package(self)
    }
}

impl Package {
    pub fn new(identifer: impl Into<String>, children: Vec<Cell>) -> Self {
        Self {
            identifer: Identifier(identifer.into()),
            children,
            runtime: Identifier("moto".to_string()),
        }
    }

    pub fn identifier_is(&self, name:  impl Into<String>) -> bool {
        self.identifer.matches(name)
    }

    pub fn name(&self) -> String {
        self.identifer.0.clone()
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.children
            .iter()
            .filter_map(|cell| match cell {
                Cell::Task(task) => Some(task.clone()),
                _ => None,
            })
            .collect()
    }


    pub fn runtimes(&self) -> Vec<Runtime> {
        self.children
            .iter()
            .filter_map(|cell| match cell {
                Cell::Runtime(runtime) => Some(runtime.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn blocks(&self) -> Vec<Block> {
        self.children
            .iter()
            .filter_map(|cell| match cell {
                Cell::Block(block) => Some(block.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn imports(&self) -> Vec<Import> {
        self.children
            .iter()
            .filter_map(|cell| match cell {
                Cell::Import(import) => Some(import.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn packages(&self) -> Vec<Package> {
        self.children
            .iter()
            .filter_map(|cell| match cell {
                Cell::Package(package) => Some(package.clone()),
                _ => None,
            })
            .collect()
    }
    
    pub fn get_task(&self, task_name: &str) -> Option<Task> {
        self.children
            .iter()
            .filter_map(|cell| match cell {
                Cell::Task(task) => {
                    if task.identifier_is(task_name) {
                        Some(task.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .next()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
///assignments are used to define variables
/// e.g `let x = 5;` or `let x = "hello world";` or `let x = true;`
/// or `let x = [1,2,3];` or `let x = {a:1, b:2};`
/// or `let x = 5 + 5;` or `let x = "hello" + "world";`
/// identifier in gray = in yellow and value in green using ascii escape codes
#[display(fmt = "\x1b[33m{identifier} \x1b[90m= \x1b[32m{value}\x1b[0m")]
pub struct Assignment {
    pub identifier: Identifier,
    pub value: Atom,
}

impl Assignment {
    pub fn new(identifier: impl Into<String>, value: impl Into<Atom>) -> Self {
        Self {
            identifier: Identifier(identifier.into()),
            value: value.into(),
        }
    }

    pub fn identifier_is(&self, name:  impl Into<String>) -> bool {
        self.identifier.matches(name)
    }

    pub fn name(&self) -> String {
        self.identifier.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
///tasks are used to define a sequence of commands
/// e.g `task greet { echo "hello world" }:shell`
/// or `task greet { print("hello world"); }:dart`
/// or `task greet { console.log("hello world"); }:js`
#[display(fmt = "task \x1b[33m{identifer}:\x1b[33m{runtime}\x1b[0m")]
pub struct Task {
    pub identifer: Identifier,
    pub body:  InterpolatedString,
    pub runtime: Identifier,
}


#[derive(Debug, Clone, PartialEq, Eq)]
/// interplated strings are a sequence of strings and variables/functions 
/// eg. `hello [:name]` or `hello [:name="world"]` or `hello [:name="world"] [:age=30]` or `hello [:name="world"] [:age=30] [:greet("world")]
/// here `[:name]` and `[:age=30]` are variables and `[:greet("world")]` is a function , rest are strings 
/// variables and functions are replaced with their values before the code is executed

pub struct InterpolatedString(pub Vec<InterpolatedStringPart>);
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum InterpolatedStringPart {
    String(String),
    Variable(Variable),
    Function(Function),
}

impl std::fmt::Display for InterpolatedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for part in &self.0 {
            write!(f, "{}", part)?;
        }
        Ok(())
    }
}

impl InterpolatedString {
    pub fn new(parts: Vec<InterpolatedStringPart>) -> Self {
        Self(parts)
    }



    pub fn raw(value: impl Into<String>) -> Self {
        Self(vec![InterpolatedStringPart::String(value.into())])
    }

    pub fn parts(&self) -> Vec<InterpolatedStringPart> {
        self.0.clone()
    }

    pub fn is_computable(&self) -> bool {
        for part in &self.0 {
            match part {
                InterpolatedStringPart::String(s) => {
                    let mut value = s.clone();
                    if value.contains("[:") {
                        return true;
                    }
                }
                InterpolatedStringPart::Variable(_) => return true,
                InterpolatedStringPart::Function(_) => return true,
            }
        }
        false
    }

    pub async fn compute(&self) -> Self {
        let mut parts = vec![];
        for part in &self.0 {
            match part {
                InterpolatedStringPart::String(value) => parts.push(InterpolatedStringPart::String(value.clone())),
                InterpolatedStringPart::Variable(variable) => {
                    let name = variable.name();
                    let default = variable.get_value_str();

                    match get_variable(&name).await {
                        Some(value) => parts.push(InterpolatedStringPart::String(value.to_string())),
                        None => parts.push(InterpolatedStringPart::String(default)),
                    }
                
                }
                InterpolatedStringPart::Function(function) => {
                  
                  }
                
            }
        }
        InterpolatedString::new(parts)
    }


    pub fn decompose(&self) -> InterpolatedString {
        let mut parts = vec![];
        for part in &self.0 {
            match part {
                InterpolatedStringPart::String(value) => {
                    let mut start = 0;
                    while let Some(left_bracket_pos) = value[start..].find("[:") {
                        let left_bracket_pos = start + left_bracket_pos;
                        if left_bracket_pos > start {
                            parts.push(InterpolatedStringPart::String(
                                value[start..left_bracket_pos].to_string(),
                            ));
                        }
                        let right_bracket_pos = value[left_bracket_pos + 2..]
                            .find(']')
                            .map(|pos| left_bracket_pos + 2 + pos)
                            .unwrap();
                        let part = &value[left_bracket_pos + 2..right_bracket_pos];
                        if let Some(left_paren_pos) = part.find('(') {
                            let name = &part[..left_paren_pos];
                            let args = &part[left_paren_pos + 1..part.len() - 1];
                            let args = args
                                .split(',')
                                .map(|arg| arg.trim())
                                .filter(|arg| !arg.is_empty())
                                .map(|arg| {
                                    if let Some(pos) = arg.find('=') {
                                        let key = &arg[..pos];
                                        let value = &arg[pos + 1..];
                                        Atom::Object( Box::new(Object{values: vec![(key.to_string(), Atom::String(value.to_string()))]}))
                                    } else {
                                        Atom::String(arg.to_string())
                                    }
                                })
                                .collect();
                            parts.push(InterpolatedStringPart::Function(Function::new(
                                name, args,
                            )));
                        } else if let Some(equals_pos) = part.find('=') {
                            let name = &part[..equals_pos];
                            let value = &part[equals_pos + 1..];
                            parts.push(InterpolatedStringPart::Variable(Variable::new(
                                name, value,
                            )));
                        } else {
                            parts.push(InterpolatedStringPart::Variable(Variable::new(
                                part, "",
                            )));
                        }
                        start = right_bracket_pos + 1;
                    }
                    if start < value.len() {
                        parts.push(InterpolatedStringPart::String(value[start..].to_string()));
                    }
                }
                part => parts.push(part.clone()),
            }
        }
        InterpolatedString::new(parts)
    }


    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn lines(&self) ->  impl Iterator<Item = InterpolatedString> {
        let tostring = self.to_string();
        let lines = tostring.split("\n").map(|x| InterpolatedStringPart::String(x.to_string())).collect();
        vec![InterpolatedString::new(lines)].into_iter()
    }
}



impl From<String> for InterpolatedString {
    fn from(value: String) -> Self {
        Self(vec![InterpolatedStringPart::String(value)])
    }
}

impl From<&str> for InterpolatedString {
    fn from(value: &str) -> Self {
        Self(vec![InterpolatedStringPart::String(value.to_string())])
    }
}

impl InterpolatedStringPart {
    pub fn to_string(&self) -> String {
        match self {
            InterpolatedStringPart::String(value) => value.clone(),
            InterpolatedStringPart::Variable(variable) => variable.to_string(),
            InterpolatedStringPart::Function(function) => function.to_string(),
        }
    }

    pub fn is_computable(&self) -> bool {
        match self {
            InterpolatedStringPart::String(s) => {
                let mut value = s.clone();
                 if value.contains("[:") {
                    return true;
                 }
                false
            }
            InterpolatedStringPart::Variable(_) => true,
            InterpolatedStringPart::Function(_) => true,
        }
    }

    pub async fn compute(&self) -> InterpolatedStringPart {
        match self {
            InterpolatedStringPart::String(value) => InterpolatedStringPart::String(value.clone()),
            InterpolatedStringPart::Variable(variable) => {
                let name = variable.name();
                let default = variable.get_value_str();

                match get_variable(&name).await {
                    Some(value) => InterpolatedStringPart::String(value.to_string()),
                    None => InterpolatedStringPart::String(default),
                }
            }
            InterpolatedStringPart::Function(function) => {
                let name = function.identifier.0.clone();
                let args = function.arguments.clone();
                InterpolatedStringPart::String(format!("[:{}({})]", name, args.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ")))
                // match get_function(&name).await {
                //     Some(value) => InterpolatedStringPart::String(value.to_string()),
                //     None => InterpolatedStringPart::String(format!("[:{}({})]", name, args.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", "))),
                // }
            }
        }
    }
}





impl Task {
    pub fn new(identifer: impl Into<String>, body: impl Into<InterpolatedString>, runtime: impl Into<String>) -> Self {
        Self {
            identifer: Identifier(identifer.into()),
            body: body.into(),
            runtime: Identifier(runtime.into()),
        }
    }

    pub fn identifier_is(&self, name:  impl Into<String>) -> bool {
        self.identifer.matches(name)
    }

    pub fn name(&self) -> String {
        self.identifer.0.clone()
    }

    pub fn runtime(&self) -> String {
        self.runtime.0.clone()
    }

    pub fn get_code(&self) -> InterpolatedString {
        self.body.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
///runtimes are used to define a runtime for a specific language
/// e.g `runtime dart {
/// let version = "3.7.0"
/// let path = "path/to/dart.exe"
///
///  task build {
///     echo "Building with dart"
///     [:path] --version
///     [:path] run [:file]
/// }:shell
/// }:moto`
#[display(fmt = "runtime \x1b[33m{identifer}:\x1b[33m{runtime}\x1b[0m")]
pub struct Runtime {
    pub identifer: Identifier,
    pub children: Vec<Cell>,
    pub runtime: Identifier,
}

impl Runtime {
    pub fn new(identifer: impl Into<String>, runtime: impl Into<String> , children: Vec<Cell>) -> Self {
        Self {
            identifer: Identifier(identifer.into()),
            children,
            runtime: Identifier(runtime.into()),
        }
    }

    pub fn identifier_is(&self, name:  impl Into<String>) -> bool {
        self.identifer.matches(name)
    }

    pub fn name(&self) -> String {
        self.identifer.0.clone()
    }

    pub fn runtime(&self) -> String {
        self.runtime.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
///blocks are used to define reusable blocks of code
/// e.g `block developerCredits { developed by incredimo for xo.rs }:text`
#[display(fmt = "block \x1b[33m{identifer}:\x1b[33m{runtime}\x1b[0m")]
pub struct Block {
    pub identifer: Identifier,
    pub body:  InterpolatedString,
    pub runtime: Identifier,
}

impl Block {
    pub fn new(identifer: impl Into<String>, body: impl Into<InterpolatedString>, runtime: impl Into<String>) -> Self {
        Self {
            identifer: Identifier(identifer.into()),
            body: body.into(),
            runtime: Identifier(runtime.into()),
        }
    }
    
    pub fn identifier_is(&self, name:  impl Into<String>) -> bool {
        self.identifer.matches(name)
    }

    pub fn name(&self) -> String {
        self.identifer.0.clone()
    }

    pub fn runtime(&self) -> String {
        self.runtime.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
///imports are used to import code from other cells
/// e.g `import "math.moto" as math`
#[display(fmt = "import [:path] as [:alias]")]
pub struct Import {
    pub path: String,
    pub alias: Identifier,
}

impl Import {
    pub fn new(path: impl Into<String>, alias: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            alias: Identifier(alias.into()),
        }
    }
}

#[derive(Debug, Clone, Display)]
///identifiers are used to define the name of a variable, task, runtime, block etc
/// e.g in `let x = 5;` x is the identifier
/// identifier always printed in bright yellow using ascii escape codes
#[display(fmt = "\x1b[33m{}\x1b[0m", "0")]
pub struct Identifier(pub String);

impl Identifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    
    pub fn matches(&self, name: impl Into<String>) -> bool {
        self.0 == name.into()
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Identifier {}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        value.0
    }
}

// impl From<Identifier> for Atom {
//     fn from(value: Identifier) -> Self {
//         Atom::Variable(Box::new(Variable {
//             identifier: value,
//             value: Atom::Null,
//         }))
//     }
// }


#[derive(Debug, Clone)]
///atoms things that return a value
///e.g `5` or `"hello world"` or `true` or `[1,2,3]` or `{a:1, b:2}` or `5 + 5` or `"hello" + "world"`
/// or `5 + x` or `x + y` or `x + 5` or `x + "hello"` or `"something" + [:x]`
pub enum Atom {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Box<Array>),
    Object(Box<Object>),
    BinaryOperation(Box<BinaryOperation>),
    // Variable(Box<Variable>),
    Function(Box<Function>),
    Null,
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Number(value) => write!(f, "{}", value),
            Atom::String(value) => write!(f, "{}", value),
            Atom::Boolean(value) => write!(f, "{}", value),
            Atom::Array(value) => write!(f, "{}", value),
            Atom::Object(value) => write!(f, "{}", value),
            Atom::BinaryOperation(value) => write!(f, "{}", value),
            // Atom::Variable(value) => write!(f, "{}", value),
            Atom::Function(value) => write!(f, "{}", value),
            Atom::Null => write!(f, "null"),
        }
    }
}



impl Atom {
    

    pub fn number(value: f64) -> Self {
        Atom::Number(value)
    }

    pub fn string(value: impl Into<String>) -> Self {
        Atom::String(value.into())
    }

    pub fn boolean(value: bool) -> Self {
        Atom::Boolean(value)
    }

    pub fn array(value: Vec<Atom>) -> Self {
        Atom::Array(Box::new(Array { values: value }))
    }

    pub fn object(value: Vec<(String, Atom)>) -> Self {
        Atom::Object(Box::new(Object { values: value }))
    }

    pub fn binary_operation(left: impl Into<Atom>, operator: impl Into<String>, right: impl Into<Atom>) -> Self {
        Atom::BinaryOperation(Box::new(BinaryOperation {
            left: left.into(),
            operator: Operator { value: operator.into() },
            right: right.into(),
        }))
    }

    // pub fn variable(identifier: impl Into<String>, default:  impl Into<Atom>) -> Self {
    //     Atom::Variable(Box::new(Variable {
    //         identifier: Identifier(identifier.into()),
    //         value: default.into(),
    //     }))
    // }
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Atom::Number(a), Atom::Number(b)) => a == b,
            (Atom::String(a), Atom::String(b)) => a == b,
            (Atom::Boolean(a), Atom::Boolean(b)) => a == b,
            (Atom::Array(a), Atom::Array(b)) => a == b,
            (Atom::Object(a), Atom::Object(b)) => a == b,
            (Atom::BinaryOperation(a), Atom::BinaryOperation(b)) => a == b,
            // (Atom::Variable(a), Atom::Variable(b)) => a == b,
            (Atom::Function(a), Atom::Function(b)) => a == b,
            (Atom::Null, Atom::Null) => true,
            _ => false,
        }
    }
}

impl Eq for Atom {}

impl From<f64> for Atom {
    fn from(value: f64) -> Self {
        Atom::Number(value)
    }
}

impl From<String> for Atom {
    fn from(value: String) -> Self {
       //if the string is a number, convert it to a number
        if let Ok(value) = value.parse::<f64>() {
            Atom::Number(value)
        } else {
            Atom::String(value)
        }
    }
}

impl From<&str> for Atom {
    fn from(value: &str) -> Self {
        Atom::String(value.to_string())
    }
}

impl From<bool> for Atom {
    fn from(value: bool) -> Self {
        Atom::Boolean(value)
    }
}

impl From<Vec<Atom>> for Atom {
    fn from(value: Vec<Atom>) -> Self {
        Atom::Array(Box::new(Array { values: value }))
    }
}

impl From<Vec<(String, Atom)>> for Atom {
    fn from(value: Vec<(String, Atom)>) -> Self {
        Atom::Object(Box::new(Object { values: value }))
    }
}

impl From<Option<Atom>> for Atom {
    fn from(value: Option<Atom>) -> Self {
        if let Some(value) = value {
            value
        } else {
            Atom::Null
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
///arrays are used to define a list of values
/// e.g `[1,2,3]` or `["hello", "world"]` or `[true, false]`
/// tailing commas are allowed and ignored
#[display(fmt = "[values.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(\", \")]")]
pub struct Array {
    pub values: Vec<Atom>,
}

impl Array {
    pub fn new(values: Vec<Atom>) -> Self {
        Self { values }
    }

    pub fn push(&mut self, value: Atom) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Option<Atom> {
        self.values.pop()
    }


    pub fn get(&self, index: usize) -> Option<&Atom> {
        self.values.get(index)
    }

    pub fn set(&mut self, index: usize, value: Atom) {
        self.values[index] = value;
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
///objects are used to define a list of key value pairs
/// e.g `{a:1, b:2}` or `{name:"incredimo", age:30}`
/// tailing commas are allowed and ignored
#[display(
    fmt = "{{values.iter().map(|(k,v)| k.to_string() + \":\" + &v.to_string()).collect::<Vec<String>>().join(\", \")}}"
)]
pub struct Object {
    pub values: Vec<(String, Atom)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
///binary operations are used to define operations between two atoms
/// e.g `5 + 5` or `"hello" + "world"` or `5 + x` or `x + y` or `x + 5` or `x + "hello"` or `"something" + [:x]`
#[display(fmt = "[:left] [:operator] [:right]")]
pub struct BinaryOperation {
    pub left: Atom,
    pub operator: Operator,
    pub right: Atom,
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
///variables are used to reference a value
/// e.g `[:name]` or `[:x=5]` or `[:x="hello"]` or `[:x=true]` or `[:x=[1,2,3]]` or `[:x={a:1, b:2}]`
/// default value is optional and can be omitted
#[display(fmt = "var {identifier}")]
pub struct Variable {
    pub identifier: Identifier,
    pub value: Atom,
}

impl Into<Cell> for Variable {
    fn into(self) -> Cell {
        Cell::Assignment(Assignment {
            identifier: self.identifier,
            value: self.value,
        })
    }
}

impl Variable {
    pub fn new(identifier: impl Into<String> , value : impl Into<Atom>) -> Self {
        Self {
            identifier: Identifier(identifier.into()),
            value: value.into(),
        }
    }

    pub fn with_default(identifier: impl Into<String>, default: Atom) -> Self {
        Self {
            identifier: Identifier(identifier.into()),
            value: default,
        }
    }

    pub fn has_default(&self) -> bool {
        self.value != Atom::Null
    }

    pub fn get_value(&self) -> Atom {
        self.value.clone()
    }

    pub fn get_value_str(&self) -> String {
        self.value.to_string()
    }

    pub fn identifier_is(&self, name:  impl Into<String>) -> bool {
        self.identifier.matches(name)
    }

    pub fn name(&self) -> String {
        self.identifier.0.clone()
    }


    pub fn set_value(&mut self, default:  impl Into<Atom>) {
        self.value = default.into();
    }

    pub fn get_identifier(&self) -> Identifier {
        self.identifier.clone()
    }

    pub fn set_identifier(&mut self, identifier: impl Into<String>) {
        self.identifier = Identifier(identifier.into());
    }
    
    pub(crate) fn get_value_or(&self, default_value: Atom) ->  Atom {
        if self.value == Atom::Null {
            default_value
        } else {
            self.value.clone()
        }
    }


}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
///functions are used to reference a function
/// e.g `[:print("hello world")]` or `[:console.log("hello world")]`
#[display(
    fmt = "[:identifier]([:arguments.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(\", \")])"
)]
pub struct Function {
    pub identifier: Identifier,
    pub arguments: Vec<Atom>,
}

impl Function {
    pub fn new(identifier: impl Into<String>, arguments: Vec<Atom>) -> Self {
        Self {
            identifier: Identifier(identifier.into()),
            arguments,
        }
    }

    pub fn name(&self) -> Identifier {
        self.identifier.clone()
    }

    pub fn set_identifier(&mut self, identifier: impl Into<String>) {
        self.identifier = Identifier(identifier.into());
    }

    pub fn args(&self) -> Vec<Atom> {
        self.arguments.clone()
    }

    pub fn set_arguments(&mut self, arguments: Vec<Atom>) {
        self.arguments = arguments;
    }
}

#[derive(Debug, Clone, Display)]
///operators are used to define operations between two atoms
/// e.g `+` or `-` or `*` or `/` or `==` or `!=` or `>` or `<` or `>=` or `<=` or `&&` or `||`

pub struct Operator {
    pub value: String,
}

impl PartialEq for Operator {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Operator {}



