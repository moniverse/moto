Sure, let's revise the example to use a different scenario. We will use an example where a task can open a browser or a specific application with an optional prefix.

# 🌈 MOTO: A Versatile Automation & Scripting System

## Overview

:moto is a versatile scripting language and automation tool designed to simplify task automation across various environments and languages. It provides a unified and intuitive syntax for defining tasks, leveraging multiple runtimes, and orchestrating complex workflows.

## Key Features

- **Multi-language Support**: :moto seamlessly integrates with multiple programming languages, including Dart, Rust, Python, JavaScript, and more. You can write code in your preferred language within a single :moto script.
- **Custom Runtimes**: Extend :moto's capabilities by defining custom runtimes for additional languages or execution environments. Specify the necessary setup and execution steps for each language.
- **Task Definition and Execution**: Define reusable tasks that encapsulate specific actions or operations. Tasks can be associated with different runtimes and executed seamlessly within the :moto script.
- **Shell Execution**: Execute shell commands and scripts using the built-in `:shell` runtime. Integrate with the underlying operating system for file manipulation, process management, and system operations.
- **Code Generation**: Generate code files dynamically using string interpolation and file output redirection. Create source files in different languages based on the script's logic.
- **Text Blocks**: Define reusable text blocks that can be referenced and interpolated within tasks or other blocks. Store common code snippets, templates, or static content for easy reuse.

## Installation

To install :moto, use the following command:

```shell
cargo install moto
```

## Usage

Here's a brief overview of the :moto language syntax:

### Variables

```moto
let name = "John"
let age = 30
let isMarried = false
let hobbies = ["reading", "coding", "gaming"]
```

### Tasks

```moto
task (please)? open (browser | [path:"chrome.exe"]) {
    let target = if ([:path]) { [:path] } else { "browser" }
    exec $target
}:shell
```

### Runtimes

```moto
task greetings_from_dart {
    void main() {
        print("Hello from Dart");
    }
}:dart  

task greet_from_rust {
    fn main() {
        println!("Hello from Rust");
    }
}:rust
```

### Custom Runtimes

```moto
runtime csharp {
    let x = 5;
    task run {
        $something = @'[:block]'@
        $something | Out-File -FilePath "./_.cs" -Encoding UTF8
        csc "./_.cs"
        ./_.exe
    }:shell
}:moto
```

### Blocks

```moto
block story {
    Once upon a time, there was a small village.
}:text

task read_story {
    echo [:story]
}:ps
```

## Parameterized Tasks with Optional Prefixes and Either-Or Conditions

With :moto, you can define tasks with optional prefixes and either-or conditions to make task invocation more flexible and intuitive.

### Example Task Definition

```moto
task (please)? open (browser | [path:"chrome.exe"]) {
    let target = if ([:path]) { [:path] } else { "browser" }
    exec $target
}:shell
```

### Example Task Invocations

1. Call the task without "please":
   ```moto
   open browser
   ```

2. Call the task with "please":
   ```moto
   please open browser
   ```

3. Call the task with a specific application path:
   ```moto
   open "firefox.exe"
   ```

4. Call the task with "please" and a specific application path:
   ```moto
   please open "firefox.exe"
   ```

### Benefits

- **Flexibility:** Supports optional prefixes and either-or conditions using `|`, allowing for more flexible task definitions.
- **Default Values:** Provides default values for parameters while allowing overrides.
- **Readability:** Enhances readability by clearly defining optional parts and default values.
- **Simplicity:** Simplifies task definitions and calls, making the scripting language more intuitive.

## Contributing

Contributions to :moto are welcome! If you encounter any issues, have suggestions for improvements, or would like to contribute new features, please open an issue or submit a pull request on the [GitHub repository](https://github.com/moniverse/moto).

