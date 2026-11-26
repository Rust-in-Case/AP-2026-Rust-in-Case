Edit only the catographer package, leave all other packages untouched.
The main function to export from cartographer is render().

I want to create a gui like this, using ratatui 0.30.2.

I only need a minimal implementation, nothing fancy, keep only one of left side pane and the right side one.
In the right side area i want to have a moving map drawn like this for a radius of 1:
 + +
+ + +
 + +

This function should take a `map: &impl Map` as a parameter. I'll need to iterate over the cells and draw different emoji based on the cell type
