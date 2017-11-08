# rust-rpn-calc
A simple 64-bit Reverse Polish Notation (RPN) calculator

#### Numbers

Entering a number places it on the stack...

    > 1 2 3

pushes 1, 2 and then 3 onto the stack, so the stack is: [1, 2, 3].

    > 4 2 3

produces the stack [4, 2, 3].


#### Commands

Entering a command's name performs some operation on the stack values...

    > 1 print

will print '1' which was on the top of the stack when 'print' was run.

    > 5 2 print

will print '2' which was on the top of the stack when 'print' was run.

    > 1 2 +

takes the top two numbers, adds them and pushes the result back onto the stack, resulting in a stack like [3].

    > 1 2 3 4 +

results in the stack [1, 2, 7].

    > 2 2 3 + *

results in the stack [10].

Use the 'help' command to print a summary of all available commands and some short descriptions
