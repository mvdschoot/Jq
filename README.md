# Jq clone
Simplified clone of the [Jq JSON processor](https://github.com/jqlang/jq). 
It is written in Rust. 
It's my first Rust project. For learning purposes and fun.

- Limited UTF support.
- Most functions are not implemented, except: 'abs', 'length', 'in', 'has', 'map'.
- Conditionals and comparisons like if-statements, '==', '<' are implemented. 
- 'Try/catch' is not implemented, but the '?' operator has the same functionality.
- Output is colored
- TODO: binding/assignment