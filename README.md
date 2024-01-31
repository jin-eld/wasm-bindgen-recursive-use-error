## Example on what triggers the `Uncaught Error: recursive use of an object detected which would lead to unsafe aliasing in rust` error with `wasm_bindgen`

Initial commit will trigger the error, the `&mut self` in the `work`
function is responsible.

The second commit removes the `mut` and also comments out modifications to
self and the error goes away.
