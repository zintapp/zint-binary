# zint

This repository concerns the small 'zint' utility that can be run inside the shell to pipe data to the [Zint](https://zint.app) application.

It merely takes its stdin and wrap it around specific Zint escape codes.

## How to use :

It has to be used from the terminal within Zint application, which is able to treat the escape codes that this binary sends.

example : `cat image.png | zint` will create an iframe (default component) that will display the image.png image file.


```

USAGE:
    zint [COMPONENT [COMPONENT_OPTIONS]]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
```
