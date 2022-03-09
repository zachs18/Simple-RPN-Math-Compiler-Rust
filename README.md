# Simple RPN Math Compiler

Takes a sequence of commands and gives a `Function` object from which can give a function pointer taking zero to six `isize`s and returning `isize`.


### Commands:

* `a`: Push the first function argument to the stack
* `b`: Push the second function argument to the stack
* `c`: Push the third function argument to the stack
* `d`: Push the fourth function argument to the stack
* `e`: Push the fifth function argument to the stack
* `f`: Push the sixth function argument to the stack
* `<positive decimal integer>`: Push value to stack
* `p<positive decimal integer>`: Push to the stack a copy of the Nth value from the top of the stack (0-indexed from the top)
* `s<positive decimal integer>`: Pop a value from the stack and set the Nth value from the top of the stack (0-indexed from the top, after the pop) to that value
* `+`: Pop two values, push their sum
* `*`: Pop two values, push their product
* `-`: Pop two values, push their difference (`a b -` gives a-b)
* `/`: Pop two values, push their quotient (`a b /` gives a/b)
* `%`: Pop two values, push their remainder (`a b %` gives a%b)


### Loops:

A loop starts with `{` and ends with `}`. Any commands (including other loops) may be inside a loop. The stack must have the same depth at the end of the loop. When execution reaches a loop, if the top value on the stack is zero, the loop will be skipped, otherwise the loop will begin. When an iteration of the loop finishes, if the value on the top of the stack is not zero, the loop will execute again, otherwise it will exit. Because loops read (but do not pop) the top value on the stack, the stack must have at least one element prior to a loop.

### Examples:

#### Exponentiation:

`a 1 b { p2 p2 * s1 1 - } p1`

Input: `4`, `3`

Output: `64`

Execution:

* Initial stack: `<empty>`

* `a` Pushes `4`. Stack: `4`
* `1` Pushes `1`. Stack: `4 1`
* `b` Pushes `3`. Stack: `4 1 3`
* `{` Top of stack is `3`, so begin loop
	- `p2` Pushes `4`. Stack: `4 1 3 4`
	- `p2` Pushes `1`. Stack: `4 1 3 4 1`
	- `*` Pops `1`, `4`, Pushes `4*1`. Stack: `4 1 3 4`
	- `s1` Pops `4`, stores at index 1. Stack: `4 4 3`
	- `1` Pushes `1`. Stack: `4 4 3 1`
	- `-` Pops `1`, `3`, Pushes `3-1`. Stack: `4 4 2`
	- `}` Top of stack is `2`, so continue loop
	- `p2` Pushes `4`. Stack: `4 4 2 4`
	- `p2` Pushes `4`. Stack: `4 4 2 4 4`
	- `*` Pops `4`, `4`, Pushes `4*4`. Stack: `4 4 2 16`
	- `s1` Pops `16`, stores at index 1. Stack: `4 16 2`
	- `1` Pushes `1`. Stack: `4 16 2 1`
	- `-` Pops `1`, `2`, Pushes `2-1`. Stack: `4 16 1`
	- `}` Top of stack is `1`, so continue loop
	- `p2` Pushes `4`. Stack: `4 16 1 4`
	- `p2` Pushes `16`. Stack: `4 16 1 4 16`
	- `*` Pops `16`, `4`, Pushes `4*16`. Stack: `4 16 1 64`
	- `s1` Pops `64`, stores at index 1. Stack: `4 64 1`
	- `1` Pushes `1`. Stack: `4 64 1 1`
	- `-` Pops `1`, `1`, Pushes `1-1`. Stack: `4 64 0`
	- `}` Top of stack is `0`, so exit loop
* `p1` Pushes `64`. Stack: `4 64 0 64`
* Top of stack is returned (`64`).