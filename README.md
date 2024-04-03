# WASM Splitter Desktop App?

Splits a web app on a per-route basis using walrus.

Steps:
- Loads route metadata
- Uses route names + no mangle to create a different binary variant for each route
- finds commonalities between all routes and saves that to a "std" binary
- all differences between routes are saved to an individual route binary
- when a page is sent down, its std + route binary is loaded
- on every navigation, we send down new route binaries and link them on the fly, along with their symbols

## This harness:

- Loads the .wasm from the web-harness
- Passes it thru walrus
- Emits an out.wasm

You need to copy output.wasm to the web-harness folder and run `python3 -m http.server 8000` to serve it.

I need to create a proper harness that serves the output.wasm and the index.html with the restitching glue.
