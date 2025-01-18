### Hello
---

This is just meant to be a simple TUI based calculator that has all the tools I need so I don't need to go look for some specific calculator on google every time I need to do anything outside of normal algebra. I basically wanted a tool like MATLAB, although I wanted to open it in under a second rather than needing to wait over a minute to just to do a copule statements and close it again.

I made a simple version [earlier](https://github.com/Whatshisname303/lex_calc) that I build without looking up any algorithms or using any dependencies. It was a lot of fun to come up with parsing from scratch, although it also meant it kind of sucked and had a few things about it that felt underdeveloped and less useful than they could be.

This version was built more with the goal of being a thought out and practical project rather than just me messing around and learning something new. It uses a really nice TUI [library](https://ratatui.rs/) as well as more formal [parser](https://en.wikipedia.org/wiki/Recursive_descent_parser). I also added more random features I came up with while building it since hey why not.

You can either clone the repo and use `cargo run` or just use `cargo install` if you don't care to use [scripts](#scripts).

### Contents
---

- [Basic Syntax](#basic-syntax)
- [Operators](#operators)
- [Variables](#variables)
- [Data Types](#data-types)
- [Functions](#functions)
- [Namespaces](#namespaces)
- [Scripts](#scripts)
- [Panels](#panels)
- [Config](#config)
- [Commands](#commands)

### Basic Syntax
---

The general syntax is about what you would expect from a calculator. You can type in expressions like `5+5` and it will print out `10`. There is also support for vectors, matrices, and functions with a few built in as well as a way to define custom functions.

Press `Enter` to input a line, and most lines will output a result which gets set to the `ans` variable. These examples show most of the different kind of expressions you can input, although there is more detail in each section.

```
a = 1 * (2 + 3)
5

[1, 0; 0, 1] * [3, 4; 5, 6]
[
	3, 4
	5, 6
]

-- lines starting with '--' are comments and do nothing

b = [5; 6] + [2; 18]
[7; 24]

std.magnitude(b)
25

-10
15

(-2)
-2
```

### Operators
---

All operators are binary (taking an input on the left and right side) except for the `-` operator which can function as a unary or a binary operator. Operators follow a precedence order you would expect, and parentheses are available to group expressions.

| Token | Precedence | Purpose        | Types                         |
| ----- | ---------- | -------------- | ----------------------------- |
| ^     | 1          | Exponents      | `any-number`                  |
| *     | 2          | Multiplication | `any-number`, `matrix-matrix` |
| /     | 2          | Division       | `number-number`               |
| +     | 3          | Addition       | `any-any(same type)`          |
| -     | 3          | Subtraction    | `any-any(same type)`, `any`   |
| =     | 4          | Assignment     | `text-any`                    |
| =>    | 4          | Alt Assignment | `any-text`                    |

### Variables
---

Variables work about as you would expect from any language with a loose type system. You can assign them, reassign them, and use them wherever you like.

```
a = [1; 2; 3]
[1; 2; 3]

a = 4
4

a + 3
7
```

There is the added `=>` operator which puts the variable name on the right side and the value on the left. This is useful since you can write an expression  and then later decide that you want to save it to a variable.

```
1 - (2 + 3 * 4) + 5
-8

=> hi
-8

1 + 2 => there
-5

hi + there
-1
```

This trick relies on the implied `ans` variable which is inserted just like most calculators when you want to continue your previous expression. Specifically if you start an expression with an operator that requires a value on the left, the parser will insert `ans` into the beginning of your expression so something like `+ 3` will be converted to `ans + 3`.

For the `-` operator, the parser will prioritize inserting `ans` and treating it as a binary operator, although you can use parentheses or something if you really want to start an expression with `-` as a unary operator

```
-- ans is not inserted
(-2 + 3)
1

-- ans is inserted (computes 1 - 2 + 3)
-2 + 3
2
```

The `ans` variable itself is initially 0 when the program starts, and it is updated whenever you execute any expression including assignment since assignment returns the value that is assigned. Commands and errors will not affect `ans`.

There is no way to delete a single variable, but you can use the `reload` command if you want to get rid of everything.

### Data Types
---

**Numbers** are just 64 bit floating point numbers. You create them... by typing a number.

**Matrices** are created within `[` and `]` using `;` to separate rows and `,` to separate values within rows.

**Vectors** are just matrices with a single column. The only real difference between matrices and vectors is in how they are displayed since vectors can be output on one line. Some of the default functions might also require a specific type to work.

```
-- creating a number
5
5

-- creating a matrix
[1, 2, 3; 4, 5, 6; 7, 8, 9]
[
	1, 2, 3
	4, 5, 6
	7, 8, 9
]

-- creating a vector
[1; 2; 3; 4]
[1; 2; 3; 4]
```

The syntax is based on MATLAB since it seemed pretty easy to type quickly and it won't be confusing switching between them. One difference though is that commas are required between values within a row. This makes it more explicit where one value ends and another begins when you use expressions as values for something like `[1; 2+3; 4]` since I don't like making rules with whitespace.

### Functions
---

You can define simple functions that can execute expressions with parameters. There isn't any huge scripting language or control flow to make complicated functions, I've mainly just found these useful for stuff like unit conversions or representing simple equations since they can be loaded from scripts and make some repetitive expressions easier to type.

You define functions starting a line with the `def` command followed by a name, a list of parameters enclosed by parentheses, a `=` and then the function body.

```
def add(a, b) = a + b

add(1, 2)
3
```

Functions contain their own scope so you don't need to worry about naming collisions when defining parameters. Functions are still able to access their parent scope though, so you can do things like call other functions or access variables that you want from within your function.

```
def add(a) = a + b

a = 1
1

b = 2
2

add(4, 5)
9

a
1

b
2

```

Also, functions and variables are stored in different places, so if you really want to, you can have both a function and a variable with the same name. Don't know why you would want this, but go ahead if you like.

```
double = 12
12

def double(double) = 2 * double

double(double(double))
48
```

One limitation is that you can't use commands from inside functions. If you want to have more complicated options to do something like execute a series of commands, use [scripts](#scripts) instead.

Since user defined functions are meant to be simple and don't have the tools to do more complicated operations, there are a chunk of default functions loaded in the `std` namespace which are just implemented in rust.

| Name            | Parameters     | Description                     |
| --------------- | -------------- | ------------------------------- |
| `std.dot`       | `vec1`, `vec2` | Vector dot product              |
| `std.cross`     | `vec1`, `vec2` | Vector cross product            |
| `std.unit`      | `vec`          | Normalized vector               |
| `std.magnitude` | `vec`          | Magnitude of vector             |
| `std.inv`       | `matrix`       | Matrix inverse                  |
| `std.det`       | `matrix`       | Matrix determinant              |
| `std.transpose` | `matrix`       | Matrix transpose                |
| `std.rref`      | `matrix`       | Matrix reduced row echelon form |
| `std.sin`       | `number`       | Sin function                    |
| `std.cos`       | `number`       | Cos function                    |
| `std.tan`       | `number`       | Tan function                    |
| `std.asin`      | `number`       | Inverse sin function            |
| `std.acos`      | `number`       | Inverse cos function            |
| `std.atan`      | `number`       | Inverse tan function            |
| `std.rad`       | `number`       | Converts degrees to radians     |
| `std.deg`       | `number`       | Converts radians to degrees     |

### Namespaces
---

The default functions are all under the `std` namespace since they have a `.` in their name and all look something like `std.dot`. You can do the same when defining your own variables and functions.

Namespaces don't have any actual meaning besides their interaction with the `use` command. This command will copy all values from the namespace to the same thing without the namespace.

The point of namespaces is just to avoid cluttering variable names with the default functions and whatever other stuff you decide to load from scripts.

```
cool.foo = 10
10

cool.foo + 2
12

use cool

foo + 2
12

cool.foo + 2
12
```

### Scripts
---

By default the program will look for a folder called config to contain all of your scripts. Any file under this directory can be loaded with `load <filename>` which will execute each line in the file one at a time.

If your config folder contains a the file `init.txt`, this will be loaded whenever you start the program.

A basic `init.txt` is included in the repo so it will be loaded if you clone the full repo. If you just build the executable with cargo install or something, then you will need to make the config folder yourself, or you can just use the program without scripts.

When splitting your config folder into multiple directories, you need to use a `.` rather than `/` to access subdirectories. So if you have the file `config/themes/cooltheme` then you should use `load themes.cooltheme` to load the script.

### Panels
---

There are currently 2 panels which you can toggle with the `panel` command which are `vars`, and `autocomplete`. They will order themselves in the order you toggle them.

To make writing config more explicit, you can also add `on` or `off` to the command rather than toggling.

```
panel vars
panel vars on
panel vars off
```

The `vars` panel just lists the current variables you have set. Setting any variable will update it at the top of the list. There is no way to scroll.

The `autocomplete` panel will show autocomplete options for variables or functions based on the current word you're typing. Pressing `tab` will complete the first option. You can also press `tab` while the panel is closed.

### Config
---

Not much is implemented here. I was going to do a few things, but they didn't seem very useful. Instead there are just config options to change your colors, although I didn't care to make the UI really pretty either, so you're basically just adjusting the syntax highlighting if you decide to mess with it. Might make things prettier in the future.

Update colors with `config theme <item> <color>` where `item` is one of the values to change from the list below and color is a hex color. For example `config theme number AA00FF`.

```
-- color options
number
identifier
unknownIdentifier
command
operator
text
inputBg
resultBg
currentBg
```

### Commands
---

Commands produce no output and are independent from the rest of the program. All the complicated ones have already been covered by this point.

| Command           | Description                                                                                                                                                                                         |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `clear`           | Clears the output without affecting the current context.                                                                                                                                            |
| `quit` or `exit`  | Exits the program.                                                                                                                                                                                  |
| `reload`          | Reloads the current context as if you just started the program. If you use `reload raw` then this will also skip the step of loading `init.txt` if it exists.                                       |
| `use <namespace>` | Copies all variables and functions while stripping the given namespace.                                                                                                                             |
| `load <script>`   | Runs a script.                                                                                                                                                                                      |
| `def ...`         | Defines a new function with the steps described [above](#functions).                                                                                                                                            |
| `config ...`      | Updates a config option.                                                                                                                                                                            |
| `show <option>`   | Shows a modal providing information based on the passed option which is either `vars`, `functions`, or `commands`.                                                                                  |
| `panel <option>`  | Toggles a panel based on the provided option which is either `vars` or `autocomplete`. You can also do `panel vars on`  or `panel vars off`  if you want to set it explicitly rather than toggling. |
