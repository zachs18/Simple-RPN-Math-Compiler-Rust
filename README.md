# Simple Math Compiler

Compiles a simple language.


### Syntax:

```
program         : item*
item            : function_item | static_item
variable        : `mut`? ident
function_item   : `fn` ident `(` (variable),* `)` block
block            : `{` statement* `}`
statement       : let_stmt | expr_stmt | loop_stmt | return_stmt
let_stmt        : `let` variable `=` expr `;`
expr_stmt       : expr `;`
loop_stmt       : `while` expr block
return_stmt     : `return` expr `;`
static_item     : `static` `atomic`? ident `=` expr `;`

expr            : assign_expr
assign_expr     : or_expr (assign_op assign_expr)*
or_expr         : and_expr (or_op or_expr)*
and_expr        : compare_expr (and_op and_expr)*
compare_expr    : add_expr (compare_op compare_expr)*
add_expr        : mul_expr (add_op add_expr)*
mul_expr        : call_expr (mul_op mul_expr)*
call_expr       : atom_expr | call_expr `(` (expr),* `)`
atom_expr       : `(` expr `)` | block | ident | literal

assign_op       : `=`
compare_op      : `>` | `>=` | `<` | `<=` | `==` | `!=`
add_op          : `+` `?`? | `-` `?`?
mul_op          : `*` `?`? | `/` `?`? | `%` `?`?
```


### Examples:

#### Exponentiation:

```  
fn exp(a, mut b) {
	let mut result = 1;
	while b > 0 {
		result = result *? a;
	}
	return result;
}
```

Input: `4`, `3`

Output: `Ok(64)`

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