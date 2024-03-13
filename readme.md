# 🌈 MOTO AUTOMATION & SCRIPTING SYSTEM

## Introduction

Welcome to the :moto language specification! :moto is more than just a scripting language; it's a versatile tool designed to simplify and automate tasks across various environments and languages. With its motto centered on versatility and simplicity, :moto empowers developers to automate repetitive tasks and orchestrate complex workflows effortlessly.

## Getting Started

### Installation

To embark on your :moto journey, install the :moto CLI using your preferred package manager:

- **Windows**: `winget install moto.moto`
- **Mac**: `brew install moto`
- **Linux**: `sudo apt-get install moto`

## Language Syntax

### Variables

Variables in :moto are declared using the `let` keyword and can hold values of any type, determined at runtime.

```moto
let name = "John"
let age = 30
let isMarried = false
let hobbies = ["reading", "coding", "gaming"]
```

### Tasks

Tasks are the core building blocks in :moto. They are defined with the `task` keyword, followed by a name and code block. They can also be associated with specific runtimes.

```moto
task hello {
    echo "Hello, $ENV{USER}!"
}:ps
```

### Runtimes

Runtimes specify the language in which a task will be executed. They're defined using the `:` operator followed by the runtime's name.

```moto
task hello {
    echo "Hello, $ENV{USER}!"
}:ps
```

:moto supports the following runtimes out of the box:
- `:shell` for shell scripting (default)
- `:dart` for Dart 
- `:rust` for Rust
- `:python` for Python
- `:javascript` for JavaScript

You can also define custom runtimes by specifying them in a `runtime` block:

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

Blocks in :moto, enclosed in curly braces `{}`, allow you to encapsulate reusable sections of code or text.

```moto
block story {
    Once upon a time, there was a small village. The villagers were afraid to go into the forest.
}:text

block story_book {
    # the story of the village 
    [:story]
}:text

task read_story {
    echo [:story_book]
}:ps
```

## Key Capabilities

### Multi-language Support

:moto allows seamless integration of multiple programming languages within a single script. You can execute code written in Dart, Rust, Python, JavaScript, and more. Each language-specific code block is enclosed within a `task` and associated with its respective runtime using the `:` syntax.

```moto
task greetings_from_dart {
    void main() {
        print("Hello from Dart");
    }
}:dart

task greet_from_rust {
    fn main() {
        println!("\x1b[31mthis is being printed from rust using \x1b[32mmoto's \x1b[33msuper smart \x1b[34mcompiler \x1b[0m");
        // ...
    }
}:rust
```

This multi-language support allows developers to leverage the strengths of different languages within a single automation script. They can choose the most suitable language for each task, whether it's Dart for mobile app development, Rust for systems programming, Python for data analysis, or JavaScript for web scripting.

### Custom Runtimes

:moto provides the flexibility to define custom runtimes using the `runtime` block. This allows developers to extend :moto's capabilities and support additional languages or execution environments.

```moto
runtime dart {
    let x = 5;
    task run {
        $something = @'[:block]'@
        $something | Out-File -FilePath "./_.dart" -Encoding UTF8
        dart run "./_.dart"
    }:shell
}:moto
```

The `runtime` block specifies the necessary setup and execution steps for each language. It defines how to generate the language-specific code file, compile or run the code, and handle any additional configuration.

Custom runtimes make it easy to integrate new languages into the :moto ecosystem. Developers can define the required commands, file extensions, and execution steps for their preferred languages, allowing seamless integration with existing tools and workflows.

### Task Definition and Execution

Tasks are the building blocks of automation in :moto. They encapsulate specific actions or operations and can be defined using the `task` keyword followed by the task name and code block.

```moto
task greet {
    start calc
}:shell

task commit {
    echo "Commiting to git"
    git commit -m "commit message"
}:shell
```

Tasks simplify automation by encapsulating reusable actions. Developers can define tasks for common operations, such as building code, running tests, deploying applications, or performing system maintenance. By abstracting complex steps into tasks, :moto makes it easier to create modular and maintainable automation scripts.

### Shell Execution

:moto provides a built-in `:shell` runtime for executing shell commands and scripts. This allows seamless integration with the underlying operating system and enables tasks like file manipulation, process management, and system operations.

```moto
task bump {
    echo "Bumping version"
    ./bump.ps1
}:ps
```

The `:shell` runtime supports executing commands in different shells, such as PowerShell (`:ps`), depending on the operating system. This flexibility ensures that :moto can adapt to various environments and leverage the power of shell scripting.

### Code Generation

:moto supports generating code files on-the-fly using string interpolation and file output redirection. This allows dynamically creating source files in different languages based on the script's logic.

```moto
runtime dart {
    let x = 5;
    task run {
        $something = @'[:block]'@
        $something | Out-File -FilePath "./_.dart" -Encoding UTF8
        dart run "./_.dart"
    }:shell
}:moto
```

In the above example, the Dart code block is generated dynamically using string interpolation (`@'[:block]'@`) and then written to a file using the `Out-File` command. This code generation capability simplifies the process of creating and executing code in different languages within the :moto script.

### Text Blocks

The `block` keyword allows defining reusable text blocks that can be referenced and interpolated within tasks or other blocks. This is useful for storing common code snippets, templates, or static content.

```moto
block developerCredits {
    // auto generated by the moto compiler 
}:text
```

Text blocks can be referenced using the `[:block_name]` syntax, making it easy to reuse and compose content across different tasks and runtimes.

## Conclusion

:moto is a powerful and versatile automation and scripting system that simplifies task automation and multi-language development. With its support for multiple runtimes, custom runtimes, task definition, shell execution, code generation, and text blocks, :moto empowers developers to streamline their workflows and boost productivity.

Whether you're automating build processes, deploying applications, or orchestrating complex workflows, :moto provides a flexible and intuitive scripting language that adapts to your needs. Embrace the power of :moto and take your automation game to the next level!

For more information and examples, visit the :moto documentation at [https://github.com/moniverse/moto](https://github.com/moniverse/moto).

Happy automating with :moto! 🚀