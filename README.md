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


## Not sure it works yet?

Runing against the docsite:
```
Input: 3426561 bytes
---------------------------
Snipping results:
    Homepage: 807 bytes, or 0.024 %
    Awesome: 349 bytes, or 0.010 %
    Deploy: 139 bytes, or 0.004 %
    Tutorial: 1060 bytes, or 0.031 %
    BlogList: 651 bytes, or 0.019 %
    PostRelease050: 373 bytes, or 0.011 %
    PostTemplate: 369 bytes, or 0.011 %
    PostFulltime: 369 bytes, or 0.011 %
    PostRelease040: 373 bytes, or 0.011 %
    PostRelease030: 373 bytes, or 0.011 %
    PostRelease020: 373 bytes, or 0.011 %
    PostRelease010: 373 bytes, or 0.011 %
    Learn: 1472 bytes, or 0.043 %
    Docs: 139 bytes, or 0.004 %
    DocsO3: 1161 bytes, or 0.034 %
    DocsO4: 139 bytes, or 0.004 %
    Docs: 139 bytes, or 0.004 %
    Err404: 824 bytes, or 0.024 %
    all: 7316 bytes, or 0.214 %
```

This leads me to believe we're not properly GC-ing the output binary - could be because the functions don't show up in the table and we don't properly trim them from the tables?

That being said, when running against the harness, I don't see them show up in the table at all.
