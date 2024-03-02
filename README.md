# Jq clone
Simplified clone of the [Jq JSON processor](https://github.com/jqlang/jq). 
It is written in Rust. 
It's my first Rust project. For learning purposes and fun.

- Limited UTF support.
- Conditionals and comparisons like if-statements, '==', '<' are implemented. 
- Output is colored
- Not implemented:
    - Most functions, except: 'abs', 'length', 'in', 'has', 'map'.
    - 'Try/catch', but the '?' operator has the same functionality.
    - Breaking out of control structures.
    - In the [Jq manual](https://jqlang.github.io/jq/manual), everything from regular expressions downwards.